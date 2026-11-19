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
    alt: bool,
}

/// Lightweight overlay + biome backdrop particle system for crafting/forge sparks & scenic weather.
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
            ambient: Vec::with_capacity(28),
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

    /// Updates and paints gentle biome-matched ambient atmosphere particles behind translucent panels.
    pub fn update_and_paint_ambient(
        &mut self,
        ctx: &egui::Context,
        rect: Rect,
        theme: ThemeId,
        time: f64,
        dt: f32,
    ) {
        if self.ambient.is_empty() {
            for idx in 0..26 {
                let x_norm = self.next_f32();
                let y_norm = self.next_f32();
                let vx_norm = (self.next_f32() - 0.35) * 0.018;
                let vy_norm = 0.008 + self.next_f32() * 0.022;
                let size = 2.0 + self.next_f32() * 2.8;
                let phase = self.next_f32() * std::f32::consts::TAU;
                self.ambient.push(AmbientMote {
                    x_norm,
                    y_norm,
                    vx_norm,
                    vy_norm,
                    size,
                    phase,
                    alt: (idx % 2) == 0,
                });
            }
        }

        let painter = ctx.layer_painter(egui::LayerId::background());
        let t = time as f32;
        let step_dt = dt.clamp(0.0, 0.05);

        for m in &mut self.ambient {
            let (dir_x, dir_y) = match theme {
                ThemeId::CherryBlossom => (0.016, m.vy_norm * 1.4),
                ThemeId::NetherCrimsonEnd | ThemeId::TrialChambers => {
                    (m.vx_norm * 0.6, -m.vy_norm * 1.5)
                }
                ThemeId::EndVoidAurora | ThemeId::DeepDarkSculk => {
                    (m.vx_norm * 0.8, -m.vy_norm * 0.55)
                }
                ThemeId::OceanMonument => (m.vx_norm * 0.4, -m.vy_norm * 1.1),
                ThemeId::PaleGarden => (0.014, (t * 0.7 + m.phase).sin() * 0.008),
                ThemeId::StarryOverworld => (m.vx_norm * 0.7, (t * 0.8 + m.phase).sin() * 0.007),
                ThemeId::DeepslateOre | ThemeId::ClassicJavaStone | ThemeId::CustomVideo => {
                    (0.012 + m.vx_norm * 0.5, -m.vy_norm * 0.35)
                }
            };
            m.x_norm = (m.x_norm + dir_x * step_dt).rem_euclid(1.0);
            m.y_norm = (m.y_norm + dir_y * step_dt).rem_euclid(1.0);

            let sway = (t * 1.4 + m.phase).sin() * 8.0;
            let pos = Pos2::new(
                rect.left() + m.x_norm * rect.width() + sway,
                rect.top() + m.y_norm * rect.height(),
            );
            let pulse = (0.45 + 0.55 * (t * 2.1 + m.phase).sin().abs()).clamp(0.2, 1.0);
            let alpha = (95.0 * pulse) as u8;

            match theme {
                ThemeId::CherryBlossom => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(255, 162, 208, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 208, 228, alpha)
                    };
                    let petal = Rect::from_center_size(pos, Vec2::new(m.size * 1.4, m.size * 0.85));
                    painter.rect_filled(petal, 1.0, col);
                }
                ThemeId::NetherCrimsonEnd => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(255, 146, 52, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 76, 68, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.0, col);
                }
                ThemeId::EndVoidAurora => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(198, 124, 255, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(92, 232, 222, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.5, col);
                }
                ThemeId::StarryOverworld => {
                    let glow = if m.alt {
                        Color32::from_rgba_unmultiplied(255, 224, 102, (130.0 * pulse) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(128, 212, 255, (110.0 * pulse) as u8)
                    };
                    painter.circle_filled(pos, m.size * 0.8, glow);
                }
                ThemeId::DeepslateOre => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(56, 214, 208, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(248, 178, 72, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::ClassicJavaStone => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(178, 236, 132, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(252, 226, 128, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.9)), 0.0, col);
                }
                ThemeId::DeepDarkSculk => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(44, 226, 218, (125.0 * pulse) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(92, 168, 220, alpha)
                    };
                    painter.circle_filled(pos, m.size * 0.75, col);
                }
                ThemeId::TrialChambers => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(252, 164, 62, (120.0 * pulse) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(72, 218, 182, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.5, col);
                }
                ThemeId::PaleGarden => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(245, 168, 56, (120.0 * pulse) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(212, 218, 228, (85.0 * pulse) as u8)
                    };
                    painter.circle_filled(pos, m.size * 0.7, col);
                }
                ThemeId::OceanMonument => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(112, 242, 232, (118.0 * pulse) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(186, 238, 252, (92.0 * pulse) as u8)
                    };
                    painter.circle_stroke(pos, m.size * 0.9, egui::Stroke::new(1.0_f32, col));
                }
                ThemeId::CustomVideo => {
                    let col = Color32::from_rgba_unmultiplied(220, 232, 248, (72.0 * pulse) as u8);
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size * 0.8)), 0.0, col);
                }
            }
        }
    }
}
