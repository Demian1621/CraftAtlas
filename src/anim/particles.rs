use eframe::egui::{self, Color32, Pos2, Rect, Vec2};

use crate::state::ThemeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleKind {
    AnvilSpark,
    XpOrb,
    CritStar,
}

#[derive(Debug, Clone)]
struct Particle {
    pos: Pos2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    kind: ParticleKind,
}

#[derive(Debug, Clone)]
struct AmbientMote {
    x_norm: f32,
    y_norm: f32,
    vx_norm: f32,
    vy_norm: f32,
    size: f32,
    phase: f32,
    variant: u8,
}

/// Lightweight overlay + biome backdrop particle system for crafting/forge sparks & subtle Minecraft weather.
#[derive(Default)]
pub struct ParticleSystem {
    items: Vec<Particle>,
    ambient: Vec<AmbientMote>,
    rng_state: u64,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            items: Vec::with_capacity(96),
            ambient: Vec::with_capacity(30),
            rng_state: 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        ((self.rng_state >> 40) as f32) / ((1u64 << 24) as f32)
    }

    pub fn spawn_anvil_burst(&mut self, origin: Pos2) {
        if self.items.len() > 220 {
            self.items.drain(0..60);
        }
        // Hot orange/gold anvil sparks + rising lime XP orbs
        for _ in 0..18 {
            let angle = -std::f32::consts::PI * self.next_f32();
            let speed = 75.0 + self.next_f32() * 145.0;
            let life = 0.55 + self.next_f32() * 0.35;
            let size = 3.0 + self.next_f32() * 3.0;
            self.items.push(Particle {
                pos: origin,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed - 35.0),
                life,
                max_life: 0.90,
                size,
                kind: ParticleKind::AnvilSpark,
            });
        }
        for _ in 0..9 {
            let ox = (self.next_f32() - 0.5) * 24.0;
            let vx = (self.next_f32() - 0.5) * 90.0;
            let vy = -(60.0 + self.next_f32() * 95.0);
            let life = 0.8 + self.next_f32() * 0.45;
            let size = 4.5 + self.next_f32() * 3.0;
            self.items.push(Particle {
                pos: origin + Vec2::new(ox, -4.0),
                vel: Vec2::new(vx, vy),
                life,
                max_life: 1.25,
                size,
                kind: ParticleKind::XpOrb,
            });
        }
    }

    #[allow(dead_code)]
    pub fn spawn_crit_sparkles(&mut self, origin: Pos2) {
        for _ in 0..10 {
            let a = self.next_f32() * std::f32::consts::TAU;
            let spd = 45.0 + self.next_f32() * 75.0;
            let life = 0.42 + self.next_f32() * 0.25;
            self.items.push(Particle {
                pos: origin,
                vel: Vec2::new(a.cos() * spd, a.sin() * spd),
                life,
                max_life: 0.67,
                size: 3.5,
                kind: ParticleKind::CritStar,
            });
        }
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn update_and_paint(&mut self, ctx: &egui::Context, dt: f32) {
        if self.items.is_empty() {
            return;
        }
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("mc_particle_layer"),
        ));

        for p in &mut self.items {
            p.life -= dt;
            match p.kind {
                ParticleKind::AnvilSpark => {
                    p.vel.y += 380.0 * dt;
                }
                ParticleKind::XpOrb => {
                    p.vel.x *= 0.96;
                    p.vel.y -= 28.0 * dt;
                }
                ParticleKind::CritStar => {
                    p.vel *= 0.92;
                }
            }
            p.pos += p.vel * dt;

            let alpha_f = (p.life / p.max_life).clamp(0.0, 1.0);
            let alpha = (alpha_f * 255.0) as u8;
            match p.kind {
                ParticleKind::AnvilSpark => {
                    let g = (140.0 + 110.0 * alpha_f) as u8;
                    let col = Color32::from_rgba_unmultiplied(255, g, 45, alpha);
                    let r = Rect::from_center_size(p.pos, Vec2::splat(p.size));
                    painter.rect_filled(r, 0.0, col);
                }
                ParticleKind::XpOrb => {
                    let col_outer = Color32::from_rgba_unmultiplied(120, 255, 68, alpha);
                    let col_core = Color32::from_rgba_unmultiplied(240, 255, 160, alpha);
                    painter.circle_filled(p.pos, p.size, col_outer);
                    painter.circle_filled(p.pos, p.size * 0.48, col_core);
                }
                ParticleKind::CritStar => {
                    let col = Color32::from_rgba_unmultiplied(120, 235, 255, alpha);
                    let r = Rect::from_center_size(p.pos, Vec2::splat(p.size));
                    painter.rect_filled(r, 1.0, col);
                }
            }
        }

        self.items.retain(|p| p.life > 0.0);
        if !self.items.is_empty() {
            ctx.request_repaint();
        }
    }

    /// Updates and paints subtle, crisp Minecraft pixel particles (petals, spores, embers, bubbles).
    pub fn update_and_paint_ambient(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        theme: ThemeId,
        bg_intensity: f32,
        time: f64,
        dt: f32,
    ) {
        if self.ambient.is_empty() {
            for idx in 0..28 {
                let x_norm = self.next_f32();
                let y_norm = self.next_f32();
                let vx_norm = (self.next_f32() - 0.35) * 0.018;
                let vy_norm = 0.008 + self.next_f32() * 0.022;
                let size = 2.0 + self.next_f32() * 2.6;
                let phase = self.next_f32() * std::f32::consts::TAU;
                self.ambient.push(AmbientMote {
                    x_norm,
                    y_norm,
                    vx_norm,
                    vy_norm,
                    size,
                    phase,
                    variant: (idx % 6) as u8,
                });
            }
        }

        let painter = ctx.layer_painter(egui::LayerId::background());
        let t = time as f32;
        let step_dt = dt.clamp(0.0, 0.05);
        let intensity_scale = (0.45 + bg_intensity.clamp(0.15, 1.0) * 0.70).clamp(0.35, 1.15);

        for m in &mut self.ambient {
            let alt = (m.variant % 2) == 0;
            let (dir_x, dir_y) = match theme {
                ThemeId::CherryBlossom => (0.016, m.vy_norm * 1.4),
                ThemeId::PaleGarden => (0.013, m.vy_norm * 1.2),
                ThemeId::NetherCrimsonEnd | ThemeId::TrialChambers => {
                    (m.vx_norm * 0.6, -m.vy_norm * 1.45)
                }
                ThemeId::EndVoidAurora | ThemeId::DeepDarkSculk => {
                    (m.vx_norm * 0.75, -m.vy_norm * 0.55)
                }
                ThemeId::OceanMonument => (m.vx_norm * 0.35, -m.vy_norm * 1.2),
                ThemeId::StarryOverworld => (m.vx_norm * 0.65, (t * 0.75 + m.phase).sin() * 0.006),
                ThemeId::DeepslateOre | ThemeId::ClassicJavaStone | ThemeId::CustomImage => {
                    (0.011 + m.vx_norm * 0.45, -m.vy_norm * 0.35)
                }
            };
            m.x_norm = (m.x_norm + dir_x * step_dt).rem_euclid(1.0);
            m.y_norm = (m.y_norm + dir_y * step_dt).rem_euclid(1.0);

            let sway = (t * 1.35 + m.phase).sin() * 7.5;
            let pos = Pos2::new(
                rect.left() + m.x_norm * rect.width() + sway,
                rect.top() + m.y_norm * rect.height(),
            );
            let pulse = (0.45 + 0.55 * (t * 2.0 + m.phase).sin().abs()).clamp(0.25, 1.0);
            let alpha = (92.0 * pulse * intensity_scale).clamp(14.0, 155.0) as u8;

            match theme {
                ThemeId::CherryBlossom => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(255, 162, 208, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 208, 228, alpha)
                    };
                    let petal = Rect::from_center_size(pos, Vec2::new(m.size * 1.45, m.size * 0.85));
                    painter.rect_filled(petal, 0.5, col);
                }
                ThemeId::PaleGarden => {
                    if m.variant == 5 {
                        // Occasional subtle warm Eyeblossom spore speck
                        let speck = Color32::from_rgba_unmultiplied(244, 168, 66, alpha);
                        painter.rect_filled(
                            Rect::from_center_size(pos, Vec2::splat(m.size * 0.85)),
                            0.0,
                            speck,
                        );
                    } else {
                        // Delicate falling Pale Oak leaf petals
                        let col = if alt {
                            Color32::from_rgba_unmultiplied(214, 220, 226, alpha)
                        } else {
                            Color32::from_rgba_unmultiplied(172, 184, 178, alpha)
                        };
                        let leaf =
                            Rect::from_center_size(pos, Vec2::new(m.size * 1.45, m.size * 0.85));
                        painter.rect_filled(leaf, 0.5, col);
                    }
                }
                ThemeId::NetherCrimsonEnd => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(255, 146, 52, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 76, 68, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.0, col);
                }
                ThemeId::TrialChambers => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(252, 158, 58, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(78, 212, 178, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.95)), 0.0, col);
                }
                ThemeId::DeepDarkSculk => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(46, 224, 214, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(110, 198, 228, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::OceanMonument => {
                    // Crisp Minecraft square pixel water bubbles rising cleanly
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(136, 240, 238, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(198, 244, 252, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::EndVoidAurora => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(198, 124, 255, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(92, 232, 222, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.0, col);
                }
                ThemeId::StarryOverworld => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(255, 224, 102, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(128, 212, 255, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::DeepslateOre => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(56, 214, 208, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(248, 178, 72, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::ClassicJavaStone => {
                    let col = if alt {
                        Color32::from_rgba_unmultiplied(178, 236, 132, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(252, 226, 128, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::CustomImage => {
                    let col = Color32::from_rgba_unmultiplied(
                        220,
                        232,
                        248,
                        (58.0 * pulse * intensity_scale).clamp(8.0, 120.0) as u8,
                    );
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.8)), 0.0, col);
                }
            }
        }
    }
}
