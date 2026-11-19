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

    /// Updates and paints 30 FPS living biome atmosphere FX + ambient motes behind translucent panels.
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
        let fx_scale = (0.45 + bg_intensity.clamp(0.15, 1.0) * 0.85).clamp(0.4, 1.25);

        paint_biome_live_fx(&painter, rect, theme, t, fx_scale);

        for m in &mut self.ambient {
            let (dir_x, dir_y) = match theme {
                ThemeId::CherryBlossom => (0.016, m.vy_norm * 1.4),
                ThemeId::NetherCrimsonEnd | ThemeId::TrialChambers => {
                    (m.vx_norm * 0.6, -m.vy_norm * 1.55)
                }
                ThemeId::EndVoidAurora | ThemeId::DeepDarkSculk => {
                    (m.vx_norm * 0.8, -m.vy_norm * 0.62)
                }
                ThemeId::OceanMonument => (m.vx_norm * 0.35, -m.vy_norm * 1.25),
                ThemeId::PaleGarden => (0.016, (t * 0.7 + m.phase).sin() * 0.008),
                ThemeId::StarryOverworld => (m.vx_norm * 0.7, (t * 0.8 + m.phase).sin() * 0.007),
                ThemeId::DeepslateOre | ThemeId::ClassicJavaStone | ThemeId::CustomImage => {
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
            let alpha = (96.0 * pulse * fx_scale).clamp(12.0, 210.0) as u8;

            match theme {
                ThemeId::CherryBlossom => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(255, 162, 208, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 208, 228, alpha)
                    };
                    let petal = Rect::from_center_size(pos, Vec2::new(m.size * 1.5, m.size * 0.85));
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
                        Color32::from_rgba_unmultiplied(
                            255,
                            224,
                            102,
                            (130.0 * pulse * fx_scale).clamp(16.0, 220.0) as u8,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(
                            128,
                            212,
                            255,
                            (110.0 * pulse * fx_scale).clamp(16.0, 200.0) as u8,
                        )
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
                    let halo_a = (38.0 * pulse * fx_scale).clamp(6.0, 90.0) as u8;
                    let core_a = (145.0 * pulse * fx_scale).clamp(18.0, 225.0) as u8;
                    let (hr, hg, hb) = if m.alt { (44, 226, 218) } else { (92, 168, 232) };
                    painter.circle_filled(
                        pos,
                        m.size * 1.45,
                        Color32::from_rgba_unmultiplied(hr, hg, hb, halo_a),
                    );
                    painter.circle_filled(
                        pos,
                        m.size * 0.65,
                        Color32::from_rgba_unmultiplied(178, 250, 255, core_a),
                    );
                }
                ThemeId::TrialChambers => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(
                            252,
                            164,
                            62,
                            (132.0 * pulse * fx_scale).clamp(16.0, 220.0) as u8,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(72, 218, 182, alpha)
                    };
                    painter.rect_filled(Rect::from_center_size(pos, Vec2::splat(m.size)), 0.5, col);
                }
                ThemeId::PaleGarden => {
                    let col = if m.alt {
                        Color32::from_rgba_unmultiplied(
                            245,
                            168,
                            56,
                            (124.0 * pulse * fx_scale).clamp(16.0, 210.0) as u8,
                        )
                    } else {
                        Color32::from_rgba_unmultiplied(
                            212,
                            218,
                            228,
                            (86.0 * pulse * fx_scale).clamp(12.0, 170.0) as u8,
                        )
                    };
                    painter.circle_filled(pos, m.size * 0.7, col);
                }
                ThemeId::OceanMonument => {
                    let r = m.size * 0.95;
                    let rim = Color32::from_rgba_unmultiplied(
                        136,
                        244,
                        246,
                        (124.0 * pulse * fx_scale).clamp(18.0, 210.0) as u8,
                    );
                    painter.circle_stroke(pos, r, egui::Stroke::new(1.1_f32, rim));
                    painter.circle_filled(
                        pos + Vec2::new(-r * 0.3, -r * 0.3),
                        (r * 0.28).max(0.8),
                        Color32::from_rgba_unmultiplied(230, 255, 255, alpha),
                    );
                }
                ThemeId::CustomImage => {
                    let col = Color32::from_rgba_unmultiplied(
                        220,
                        232,
                        248,
                        (68.0 * pulse * fx_scale).clamp(8.0, 150.0) as u8,
                    );
                    painter.circle_filled(pos, m.size * 0.65, col);
                }
            }
        }
    }
}

fn rel_pos(rect: Rect, nx: f32, ny: f32) -> Pos2 {
    Pos2::new(rect.left() + nx * rect.width(), rect.top() + ny * rect.height())
}

fn draw_quad_beam(
    painter: &egui::Painter,
    top_left: Pos2,
    top_right: Pos2,
    bot_right: Pos2,
    bot_left: Pos2,
    color: Color32,
) {
    painter.add(egui::Shape::convex_polygon(
        vec![top_left, top_right, bot_right, bot_left],
        color,
        egui::Stroke::NONE,
    ));
}

/// Renders rich, biome-authentic 30 FPS living effects (sonic rings, god-rays, mist, caustics, aurora).
fn paint_biome_live_fx(
    painter: &egui::Painter,
    rect: Rect,
    theme: ThemeId,
    t: f32,
    fx_scale: f32,
) {
    match theme {
        ThemeId::DeepDarkSculk => {
            let portal = rel_pos(rect, 0.50, 0.37);
            let max_r = rect.width().max(rect.height()) * 0.42;
            for ring in 0..3 {
                let phase = (t * 0.22 + (ring as f32) / 3.0).fract();
                let radius = 24.0 + phase * max_r;
                let fade = (1.0 - phase).powf(1.35);
                let stroke_a = (88.0 * fade * fx_scale).clamp(0.0, 140.0) as u8;
                let fill_a = (16.0 * fade * fx_scale).clamp(0.0, 32.0) as u8;
                if fill_a > 1 {
                    painter.circle_filled(
                        portal,
                        radius * 0.42,
                        Color32::from_rgba_unmultiplied(36, 218, 224, fill_a),
                    );
                }
                if stroke_a > 2 {
                    painter.circle_stroke(
                        portal,
                        radius,
                        egui::Stroke::new(2.2_f32, Color32::from_rgba_unmultiplied(48, 232, 228, stroke_a)),
                    );
                }
            }
            for (idx, &(nx, ny)) in [
                (0.22_f32, 0.62_f32),
                (0.76, 0.58),
                (0.42, 0.72),
                (0.62, 0.78),
                (0.34, 0.44),
            ]
            .iter()
            .enumerate()
            {
                let pulse = (0.35 + 0.65 * (t * 1.45 + idx as f32 * 1.37).sin().abs()).clamp(0.1, 1.0);
                let p = rel_pos(rect, nx, ny);
                let a = (32.0 * pulse * fx_scale).clamp(2.0, 64.0) as u8;
                painter.circle_filled(p, 28.0 + pulse * 14.0, Color32::from_rgba_unmultiplied(38, 222, 214, a));
            }
        }
        ThemeId::TrialChambers => {
            for ray in 0..4 {
                let rf = ray as f32;
                let sway = (t * 0.38 + rf * 1.4).sin() * 0.035;
                let top_x = 0.66 + rf * 0.048 + sway * 0.4;
                let bot_x = 0.22 + rf * 0.145 + sway;
                let w_top = 0.036;
                let w_bot = 0.11;
                let pulse = 0.55 + 0.45 * (t * 0.85 + rf * 1.9).sin();
                let a = (28.0 * pulse * fx_scale).clamp(4.0, 56.0) as u8;
                draw_quad_beam(
                    painter,
                    rel_pos(rect, top_x, 0.0),
                    rel_pos(rect, top_x + w_top, 0.0),
                    rel_pos(rect, bot_x + w_bot, 1.0),
                    rel_pos(rect, bot_x, 1.0),
                    Color32::from_rgba_unmultiplied(255, 206, 128, a),
                );
            }
            for (idx, &(nx, ny)) in [
                (0.58_f32, 0.66_f32),
                (0.66, 0.64),
                (0.74, 0.68),
                (0.62, 0.73),
            ]
            .iter()
            .enumerate()
            {
                let flick = (0.55
                    + 0.28 * (t * 5.2 + idx as f32 * 2.1).sin()
                    + 0.17 * (t * 9.3 + idx as f32).cos())
                .clamp(0.25, 1.0);
                let p = rel_pos(rect, nx, ny);
                painter.circle_filled(
                    p,
                    38.0 * flick + 18.0,
                    Color32::from_rgba_unmultiplied(
                        255,
                        148,
                        48,
                        (44.0 * flick * fx_scale).clamp(6.0, 76.0) as u8,
                    ),
                );
            }
        }
        ThemeId::PaleGarden => {
            // Drifting parallax mist ribbons across the lower forest
            for band in 0..3 {
                let bf = band as f32;
                let speed = 0.022 + bf * 0.015;
                let ny = 0.66 + bf * 0.10;
                let drift = (t * speed + bf * 0.33).fract();
                for dup in [-1.0_f32, 0.0_f32, 1.0_f32] {
                    let cx = rect.left() + (drift + dup) * rect.width();
                    let cy = rect.top() + ny * rect.height();
                    let r = Rect::from_center_size(
                        Pos2::new(cx, cy),
                        Vec2::new(rect.width() * 0.68, rect.height() * (0.14 + bf * 0.03)),
                    );
                    let a = ((22.0 - bf * 3.0) * fx_scale).clamp(4.0, 38.0) as u8;
                    painter.rect_filled(r, rect.height() * 0.07, Color32::from_rgba_unmultiplied(196, 206, 222, a));
                }
            }
            // Blinking Creaking amber eyes peeking out of shadowed trees
            for (idx, &(nx, ny)) in [
                (0.24_f32, 0.48_f32),
                (0.58, 0.43),
                (0.78, 0.52),
            ]
            .iter()
            .enumerate()
            {
                let cycle = (t * 0.24 + idx as f32 * 0.37).fract();
                let open = if !(0.12..=0.72).contains(&cycle) {
                    0.0
                } else if (0.42..=0.49).contains(&cycle) {
                    0.12 // quick eerie blink
                } else {
                    let fade_in = ((cycle - 0.12) / 0.10).clamp(0.0, 1.0);
                    let fade_out = ((0.72 - cycle) / 0.14).clamp(0.0, 1.0);
                    fade_in.min(fade_out)
                };
                if open > 0.02 {
                    let c = rel_pos(rect, nx, ny);
                    let eye_a = (210.0 * open * fx_scale).clamp(0.0, 240.0) as u8;
                    let halo_a = (46.0 * open * fx_scale).clamp(0.0, 80.0) as u8;
                    let col = Color32::from_rgba_unmultiplied(255, 166, 52, eye_a);
                    let halo = Color32::from_rgba_unmultiplied(255, 146, 38, halo_a);
                    for side in [-4.5_f32, 4.5_f32] {
                        let ep = c + Vec2::new(side, 0.0);
                        painter.circle_filled(ep, 6.0, halo);
                        painter.rect_filled(Rect::from_center_size(ep, Vec2::new(3.2, 2.2)), 0.5, col);
                    }
                }
            }
            // Subtle warm Eyeblossom ground pulses
            for (idx, &(nx, ny)) in [(0.32_f32, 0.84_f32), (0.64, 0.87), (0.48, 0.82)].iter().enumerate() {
                let p = rel_pos(rect, nx, ny);
                let pulse = (0.3 + 0.7 * (t * 1.2 + idx as f32 * 1.7).sin().abs()).clamp(0.1, 1.0);
                painter.circle_filled(
                    p,
                    18.0 + pulse * 8.0,
                    Color32::from_rgba_unmultiplied(
                        246,
                        162,
                        56,
                        (26.0 * pulse * fx_scale).clamp(2.0, 52.0) as u8,
                    ),
                );
            }
        }
        ThemeId::OceanMonument => {
            // 6 underwater sun caustic beams sweeping slowly from surface
            for ray in 0..6 {
                let rf = ray as f32;
                let sway = (t * 0.42 + rf * 1.15).sin() * 0.04;
                let top_x = 0.08 + rf * 0.145 + sway * 0.5;
                let bot_x = top_x + 0.14 + sway;
                let pulse = 0.50 + 0.50 * (t * 1.1 + rf * 1.5).sin();
                let a = (24.0 * pulse * fx_scale).clamp(4.0, 48.0) as u8;
                draw_quad_beam(
                    painter,
                    rel_pos(rect, top_x, 0.0),
                    rel_pos(rect, top_x + 0.045, 0.0),
                    rel_pos(rect, bot_x + 0.095, 0.92),
                    rel_pos(rect, bot_x, 0.92),
                    Color32::from_rgba_unmultiplied(92, 232, 240, a),
                );
            }
            // Sea Lantern doorway & crown pulse
            for (nx, ny, base_r) in [(0.50_f32, 0.63_f32, 46.0_f32), (0.50, 0.18, 28.0)] {
                let pulse = (0.55 + 0.45 * (t * 1.65).sin()).clamp(0.2, 1.0);
                painter.circle_filled(
                    rel_pos(rect, nx, ny),
                    base_r + pulse * 14.0,
                    Color32::from_rgba_unmultiplied(
                        94,
                        244,
                        236,
                        (38.0 * pulse * fx_scale).clamp(4.0, 72.0) as u8,
                    ),
                );
            }
            // 2 distant Guardian silhouettes gliding across mid-depths
            for g in 0..2 {
                let gf = g as f32;
                let x = if g == 0 {
                    (t * 0.032 + 0.15).fract()
                } else {
                    1.0 - (t * 0.025 + 0.60).fract()
                };
                let y = 0.32 + gf * 0.22 + (t * 1.1 + gf * 2.0).sin() * 0.018;
                let p = rel_pos(rect, x, y);
                let tail_dir = if g == 0 { -1.0_f32 } else { 1.0_f32 };
                let body_a = (92.0 * fx_scale).clamp(18.0, 140.0) as u8;
                let eye_a = (165.0 * fx_scale).clamp(24.0, 210.0) as u8;
                painter.rect_filled(
                    Rect::from_center_size(p, Vec2::new(13.0, 10.0)),
                    2.0,
                    Color32::from_rgba_unmultiplied(28, 72, 78, body_a),
                );
                let tail_wag = (t * 5.0 + gf * 1.8).sin() * 3.0;
                painter.rect_filled(
                    Rect::from_center_size(
                        p + Vec2::new(tail_dir * 9.5, tail_wag),
                        Vec2::new(7.0, 4.0),
                    ),
                    1.0,
                    Color32::from_rgba_unmultiplied(36, 92, 98, body_a),
                );
                painter.circle_filled(
                    p + Vec2::new(-tail_dir * 2.5, 0.0),
                    2.1,
                    Color32::from_rgba_unmultiplied(255, 118, 92, eye_a),
                );
            }
        }
        ThemeId::EndVoidAurora => {
            for ribbon in 0..2 {
                let rf = ribbon as f32;
                let base_y = 0.10 + rf * 0.09;
                let segs = 12usize;
                for s in 0..segs {
                    let u0 = s as f32 / segs as f32;
                    let u1 = (s + 1) as f32 / segs as f32;
                    let y0 = base_y + (t * 0.55 + u0 * 5.2 + rf * 1.8).sin() * 0.045;
                    let y1 = base_y + (t * 0.55 + u1 * 5.2 + rf * 1.8).sin() * 0.045;
                    let h = 0.09 + 0.03 * (t * 0.9 + u0 * 4.0).cos();
                    let wave_alpha = 0.5 + 0.5 * (t * 0.7 + u0 * 3.5 + rf).sin();
                    let a = (24.0 * wave_alpha * fx_scale).clamp(3.0, 44.0) as u8;
                    let col = if ribbon == 0 {
                        Color32::from_rgba_unmultiplied(92, 236, 214, a)
                    } else {
                        Color32::from_rgba_unmultiplied(198, 120, 255, a)
                    };
                    draw_quad_beam(
                        painter,
                        rel_pos(rect, u0, y0),
                        rel_pos(rect, u1, y1),
                        rel_pos(rect, u1, y1 + h),
                        rel_pos(rect, u0, y0 + h),
                        col,
                    );
                }
            }
        }
        ThemeId::StarryOverworld => {
            // Twinkling fixed sky stars + periodic shooting star every ~5.5s
            for (idx, &(nx, ny)) in [
                (0.14_f32, 0.12_f32),
                (0.28, 0.08),
                (0.46, 0.16),
                (0.63, 0.09),
                (0.81, 0.14),
                (0.90, 0.22),
            ]
            .iter()
            .enumerate()
            {
                let tw = 0.35 + 0.65 * (t * (1.8 + idx as f32 * 0.4) + idx as f32).sin().abs();
                painter.circle_filled(
                    rel_pos(rect, nx, ny),
                    1.6 + tw * 0.8,
                    Color32::from_rgba_unmultiplied(
                        216,
                        236,
                        255,
                        (145.0 * tw * fx_scale).clamp(16.0, 215.0) as u8,
                    ),
                );
            }
            let cycle = (t / 5.5).fract();
            if cycle < 0.18 {
                let p = cycle / 0.18;
                let head = rel_pos(rect, 0.18 + p * 0.56, 0.08 + p * 0.20);
                let tail = rel_pos(
                    rect,
                    (0.18 + p * 0.56 - 0.10).max(0.0),
                    (0.08 + p * 0.20 - 0.036).max(0.0),
                );
                let a = (195.0 * (1.0 - p) * fx_scale).clamp(0.0, 230.0) as u8;
                painter.line_segment(
                    [tail, head],
                    egui::Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(190, 235, 255, a)),
                );
                painter.circle_filled(head, 2.4, Color32::from_rgba_unmultiplied(245, 252, 255, a));
            }
        }
        ThemeId::CherryBlossom => {
            for ray in 0..3 {
                let rf = ray as f32;
                let x0 = 0.06 + rf * 0.11 + (t * 0.28 + rf).sin() * 0.025;
                let x1 = x0 + 0.26;
                let a = ((18.0 + 6.0 * (t * 0.6 + rf).sin()) * fx_scale).clamp(3.0, 34.0) as u8;
                draw_quad_beam(
                    painter,
                    rel_pos(rect, x0, 0.0),
                    rel_pos(rect, x0 + 0.055, 0.0),
                    rel_pos(rect, x1 + 0.10, 0.95),
                    rel_pos(rect, x1, 0.95),
                    Color32::from_rgba_unmultiplied(255, 198, 214, a),
                );
            }
        }
        ThemeId::DeepslateOre => {
            for (idx, &(nx, ny, is_cyan)) in [
                (0.20_f32, 0.56_f32, false),
                (0.78, 0.44, true),
                (0.62, 0.72, true),
            ]
            .iter()
            .enumerate()
            {
                let pulse = 0.4 + 0.6 * (t * 1.35 + idx as f32 * 2.0).sin().abs();
                let a = (30.0 * pulse * fx_scale).clamp(4.0, 62.0) as u8;
                let col = if is_cyan {
                    Color32::from_rgba_unmultiplied(56, 224, 212, a)
                } else {
                    Color32::from_rgba_unmultiplied(248, 178, 72, a)
                };
                painter.circle_filled(rel_pos(rect, nx, ny), 34.0 + pulse * 14.0, col);
            }
        }
        ThemeId::NetherCrimsonEnd => {
            let glow_pulse = 0.55 + 0.45 * (t * 1.4).sin();
            let a = (28.0 * glow_pulse * fx_scale).clamp(4.0, 52.0) as u8;
            let lava_band = Rect::from_min_max(
                rel_pos(rect, 0.0, 0.78),
                rel_pos(rect, 1.0, 1.0),
            );
            painter.rect_filled(lava_band, 0.0, Color32::from_rgba_unmultiplied(255, 92, 42, a));
        }
        ThemeId::ClassicJavaStone => {
            let sun = 0.5 + 0.5 * (t * 0.7).sin();
            let a = (20.0 * sun * fx_scale).clamp(3.0, 38.0) as u8;
            painter.circle_filled(
                rel_pos(rect, 0.72, 0.16),
                rect.height() * 0.24,
                Color32::from_rgba_unmultiplied(255, 232, 164, a),
            );
        }
        ThemeId::CustomImage => {}
    }
}
