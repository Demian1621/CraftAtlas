use eframe::egui::{self, Color32};
use std::path::PathBuf;

use crate::anim::ToastQueue;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::Language;
use crate::data::seed_engine::Dimension;
use crate::i18n::{t, UiKey};
use crate::state::AppState;
pub(super) fn default_export_png_path(seed: i64, dim: Dimension) -> Option<PathBuf> {
    let xdg_pic = std::env::var_os("XDG_PICTURES_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.exists());
    let dir = if let Some(p) = xdg_pic {
        p
    } else {
        let base = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)?;
        let pictures = base.join("Pictures");
        let imagenes = base.join("Imágenes");
        if pictures.exists() {
            pictures
        } else if imagenes.exists() {
            imagenes
        } else {
            base
        }
    };
    let dname = match dim {
        Dimension::Overworld => "overworld",
        Dimension::Nether => "nether",
        Dimension::TheEnd => "end",
    };
    Some(dir.join(format!("seed_{}_{}.png", seed, dname)))
}

pub(super) fn handle_dropped_skin_png(
    ctx: &egui::Context,
    state: &mut AppState,
    audio: &AudioEngine,
    toasts: &mut ToastQueue,
    lang: Language,
) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
    if dropped.is_empty() {
        return;
    }
    for f in dropped {
        let is_png = f
            .path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("png"))
            || f.name.to_ascii_lowercase().ends_with(".png");
        if !is_png {
            continue;
        }
        let res = if let Some(ref path) = f.path {
            state.apply_custom_skin_file(ctx, path)
        } else if let Some(ref bytes) = f.bytes {
            state.apply_custom_skin_bytes(ctx, bytes)
        } else {
            continue;
        };
        match res {
            Ok(()) => {
                audio.play(SoundCue::XpOrb);
                toasts.push(
                    t(lang, UiKey::PlayerSkinSavedToastTitle),
                    t(lang, UiKey::PlayerSkinCustomBadge),
                    "compass",
                    Color32::from_rgb(80, 215, 130),
                );
            }
            Err(err) => {
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::PlayerSkinErrorToastTitle),
                    err,
                    "barrier",
                    Color32::from_rgb(235, 85, 85),
                );
            }
        }
        break;
    }
}

