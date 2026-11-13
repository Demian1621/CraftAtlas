use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;

/// Semantic UI sound cues synced cleanly to Minecraft `.minecraft/assets` `.ogg` or gentle PCM synth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCue {
    Click,
    AnvilUse,
    XpOrb,
    LevelUp,
    Toast,
}

impl SoundCue {
    fn key(self) -> &'static str {
        match self {
            SoundCue::Click => "click",
            SoundCue::AnvilUse => "anvil_use",
            SoundCue::XpOrb => "orb",
            SoundCue::LevelUp => "levelup",
            SoundCue::Toast => "toast",
        }
    }

    /// Per-cue gain multiplier so harsh strikes stay balanced relative to UI clicks.
    pub fn volume_scale(self) -> f32 {
        match self {
            // Anvil strike .ogg has intense metallic peaks; linear 0.24 (~ -12.4 dB)
            // cuts perceived loudness cleanly below UI clicks and XP chimes.
            SoundCue::AnvilUse => 0.24,
            SoundCue::LevelUp => 0.60,
            _ => 1.0,
        }
    }
}

/// Computes final playback volume for `cue` given master `volume` in `[0.0, 1.0]`.
pub fn effective_cue_volume(volume: f32, cue: SoundCue) -> f32 {
    (volume.clamp(0.0, 1.0) * cue.volume_scale()).clamp(0.0, 1.0)
}

struct AudioCmd {
    cue: SoundCue,
    volume: f32,
}

/// Non-blocking async audio engine. Never stalls UI frames and degrades silently if no audio hardware exists.
pub struct AudioEngine {
    tx: Option<Sender<AudioCmd>>,
    pub enabled: bool,
    pub volume: f32,
}

impl AudioEngine {
    pub fn new(enabled: bool, volume: f32, sound_paths: HashMap<String, PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel::<AudioCmd>();
        thread::Builder::new()
            .name("mc-audio-worker".into())
            .spawn(move || audio_worker_loop(rx, sound_paths))
            .ok();

        Self {
            tx: Some(tx),
            enabled,
            volume: volume.clamp(0.0, 1.0),
        }
    }

    pub fn play(&self, cue: SoundCue) {
        if !self.enabled || self.volume <= 0.005 {
            return;
        }
        if let Some(ref tx) = self.tx {
            let _ = tx.send(AudioCmd {
                cue,
                volume: self.volume,
            });
        }
    }
}

fn audio_worker_loop(rx: mpsc::Receiver<AudioCmd>, sound_paths: HashMap<String, PathBuf>) {
    let Ok((_stream, handle)) = OutputStream::try_default() else {
        // Headless environment or missing audio output device: drain quietly.
        while rx.recv().is_ok() {}
        return;
    };

    while let Ok(cmd) = rx.recv() {
        let Ok(sink) = Sink::try_new(&handle) else {
            continue;
        };
        sink.set_volume(effective_cue_volume(cmd.volume, cmd.cue));

        let played_ogg = if let Some(path) = sound_paths.get(cmd.cue.key()) {
            if let Ok(file) = File::open(path) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    sink.append(source.amplify(0.8));
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !played_ogg {
            sink.append(synth_cue(cmd.cue));
        }

        sink.detach();
    }
}

/// Pure-Rust gentle retro chime fallback when local `.minecraft` `.ogg` files are unavailable.
fn synth_cue(cue: SoundCue) -> SamplesBuffer<f32> {
    let sr = 44100u32;
    let notes: &[(f32, f32)] = match cue {
        SoundCue::Click => &[(880.0, 0.032)],
        SoundCue::AnvilUse => &[(210.0, 0.04), (495.0, 0.14)],
        SoundCue::XpOrb => &[(987.77, 0.05), (1318.51, 0.08)],
        SoundCue::LevelUp => &[
            (523.25, 0.06),
            (659.25, 0.06),
            (783.99, 0.07),
            (1046.50, 0.14),
        ],
        SoundCue::Toast => &[(659.25, 0.07), (987.77, 0.12)],
    };

    let total_samples: usize = notes.iter().map(|(_, dur)| ((sr as f32) * dur) as usize).sum();
    let mut out = Vec::with_capacity(total_samples);

    for &(freq, dur) in notes {
        let count = ((sr as f32) * dur) as usize;
        for i in 0..count {
            let t = i as f32 / sr as f32;
            let env = (1.0 - (i as f32 / count as f32)).powf(1.8);
            // Soft triangle/sine blend for warm non-harsh chime
            let wave = (t * freq * std::f32::consts::TAU).sin() * 0.7
                + (t * freq * 2.0 * std::f32::consts::TAU).sin() * 0.18;
            out.push(wave * env * 0.25);
        }
    }

    SamplesBuffer::new(1, sr, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anvil_and_levelup_sounds_are_attenuated() {
        let click = effective_cue_volume(0.8, SoundCue::Click);
        let anvil = effective_cue_volume(0.8, SoundCue::AnvilUse);
        let levelup = effective_cue_volume(0.8, SoundCue::LevelUp);
        assert!((click - 0.8).abs() < 1e-6);
        assert!((anvil - 0.192).abs() < 1e-6);
        assert!((levelup - 0.48).abs() < 1e-6);
        assert!((anvil / click - 0.24).abs() < 1e-6);
        assert_eq!(effective_cue_volume(1.5, SoundCue::AnvilUse), 0.24);
        assert_eq!(effective_cue_volume(-0.2, SoundCue::AnvilUse), 0.0);
    }
}
