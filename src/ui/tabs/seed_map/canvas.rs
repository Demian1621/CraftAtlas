use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, RichText, Stroke, Vec2};

use crate::anim::ToastQueue;
use crate::audio::SoundCue;
use crate::data::models::Language;
use crate::data::seed_engine::{
    is_slime_chunk, Dimension, SeedMapEngine, StructureKind, StructureMarker, TILE_PX,
};
use crate::i18n::{t, UiKey};
use crate::state::{AppState, MapContextMenuState, WAYPOINT_ICON_PRESETS};
use crate::ui::theme::{paint_mc_frame, paint_mc_slot, show_mc_tooltip_at, SlotStyle};
use crate::ui::RenderContext;

use super::{format_tp_command, format_tp_xyz};
pub(super) fn render_map_viewport(
    ui: &mut egui::Ui,
    state: &mut AppState,
    engine: &mut SeedMapEngine,
    rcx: &RenderContext<'_>,
    toasts: &mut ToastQueue,
) {
    let RenderContext {
        palette,
        atlas,
        audio,
        ..
    } = *rcx;
    let lang = state.prefs.language;
    let avail = ui.available_size();
    let hud_h = 52.0;
    let canvas_h = (avail.y - hud_h - 6.0).max(220.0);
    let (canvas_rect, resp) = ui.allocate_exact_size(
        Vec2::new(avail.x, canvas_h),
        egui::Sense::click_and_drag(),
    );

    paint_mc_frame(ui.painter(), canvas_rect, palette, Some(palette.border_strong));

    let view_rect = canvas_rect.shrink(2.0);
    let painter = ui.painter().with_clip_rect(view_rect);

    // 1. Dark dimension canvas background
    let dim_bg = match state.map_dim {
        Dimension::Overworld => Color32::from_rgb(16, 28, 36),
        Dimension::Nether => Color32::from_rgb(38, 12, 14),
        Dimension::TheEnd => Color32::from_rgb(12, 8, 22),
    };
    painter.rect_filled(view_rect, 2.0, dim_bg);

    // 2. Drag Pan (never triggers click!)
    if resp.dragged() {
        let delta = resp.drag_delta();
        let bpp = state.map_blocks_per_px as f64;
        state.map_center_x -= (delta.x as f64) * bpp;
        state.map_center_z -= (delta.y as f64) * bpp;
        state.map_target_center_x = state.map_center_x;
        state.map_target_center_z = state.map_center_z;
    }

    // 3. Smooth Cursor-Anchored Zoom on Scroll
    let hovered = resp.hovered();
    let pointer_pos = ui.ctx().pointer_latest_pos().filter(|p| view_rect.contains(*p));
    if hovered {
        let scroll_y = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if scroll_y.abs() > 0.5 {
            let factor = (-scroll_y * 0.0028).exp();
            let old_bpp = state.map_target_bpp;
            let new_bpp = (old_bpp * factor).clamp(0.30, 160.0);
            if let Some(p) = pointer_pos {
                let dx_px = (p.x - view_rect.center().x) as f64;
                let dy_px = (p.y - view_rect.center().y) as f64;
                let world_x = state.map_target_center_x + dx_px * (old_bpp as f64);
                let world_z = state.map_target_center_z + dy_px * (old_bpp as f64);
                state.map_target_center_x = world_x - dx_px * (new_bpp as f64);
                state.map_target_center_z = world_z - dy_px * (new_bpp as f64);
            }
            state.map_target_bpp = new_bpp;
        }
    }

    let bpp = state.map_blocks_per_px;
    let center_x = state.map_center_x;
    let center_z = state.map_center_z;
    let half_w_blocks = (view_rect.width() * 0.5 * bpp) as f64;
    let half_h_blocks = (view_rect.height() * 0.5 * bpp) as f64;
    let min_bx = center_x - half_w_blocks;
    let max_bx = center_x + half_w_blocks;
    let min_bz = center_z - half_h_blocks;
    let max_bz = center_z + half_h_blocks;

    let world_to_screen = |wx: f64, wz: f64| -> Pos2 {
        Pos2::new(
            view_rect.center().x + ((wx - center_x) / (bpp as f64)) as f32,
            view_rect.center().y + ((wz - center_z) / (bpp as f64)) as f32,
        )
    };
    let screen_to_world = |pos: Pos2| -> (f64, f64) {
        (
            center_x + ((pos.x - view_rect.center().x) as f64) * (bpp as f64),
            center_z + ((pos.y - view_rect.center().y) as f64) * (bpp as f64),
        )
    };

    // 4. Async Cubiomes Terrain Tile Pyramid
    if state.map_layers.biomes {
        let scale = pick_cubiomes_scale(bpp);
        let tile_blocks = (TILE_PX as i32 * scale) as f64;
        let tx0 = (min_bx / tile_blocks).floor() as i32;
        let tx1 = (max_bx / tile_blocks).ceil() as i32;
        let tz0 = (min_bz / tile_blocks).floor() as i32;
        let tz1 = (max_bz / tile_blocks).ceil() as i32;

        // Bound iteration safely on extreme aspect ratios and prioritize center-out tile streaming
        if (tx1 - tx0) <= 48 && (tz1 - tz0) <= 48 {
            let center_tx = (state.map_center_x / tile_blocks) - 0.5;
            let center_tz = (state.map_center_z / tile_blocks) - 0.5;
            let mut order: Vec<(f64, i32, i32)> =
                Vec::with_capacity(((tx1 - tx0 + 1) * (tz1 - tz0 + 1)).max(0) as usize);
            for tz in tz0..=tz1 {
                for tx in tx0..=tx1 {
                    let dx = (tx as f64) - center_tx;
                    let dz = (tz as f64) - center_tz;
                    order.push((dx * dx + dz * dz, tx, tz));
                }
            }
            order.sort_by(|a, b| a.0.total_cmp(&b.0));

            for (_, tx, tz) in order {
                let bx0 = (tx as f64) * tile_blocks;
                let bz0 = (tz as f64) * tile_blocks;
                let p0 = world_to_screen(bx0, bz0);
                let p1 = world_to_screen(bx0 + tile_blocks, bz0 + tile_blocks);
                let trect = Rect::from_min_max(p0, p1);
                if let Some((tex, uv)) =
                    engine.lookup_tile_or_fallback(state.map_dim, scale, state.map_style, tx, tz)
                {
                    painter.image(tex.id(), trect, uv, Color32::WHITE);
                } else {
                    painter.rect_filled(trect.shrink(0.5), 0.0, Color32::from_black_alpha(45));
                }
            }
        }
    }

    // 5. Slime Chunks Overlay (Overworld only when zoomed in cleanly so zero clutter!)
    if state.map_layers.slime_chunks && state.map_dim == Dimension::Overworld && bpp <= 4.0 {
        let cx0 = (min_bx / 16.0).floor() as i32;
        let cx1 = (max_bx / 16.0).ceil() as i32;
        let cz0 = (min_bz / 16.0).floor() as i32;
        let cz1 = (max_bz / 16.0).ceil() as i32;
        if (cx1 - cx0) <= 80 && (cz1 - cz0) <= 60 {
            for cz in cz0..=cz1 {
                for cx in cx0..=cx1 {
                    if is_slime_chunk(engine.seed, cx, cz) {
                        let p0 = world_to_screen((cx * 16) as f64, (cz * 16) as f64);
                        let p1 = world_to_screen(((cx + 1) * 16) as f64, ((cz + 1) * 16) as f64);
                        let crect = Rect::from_min_max(p0, p1);
                        painter.rect_filled(
                            crect,
                            0.0,
                            Color32::from_rgba_unmultiplied(64, 240, 64, 82),
                        );
                        painter.rect_stroke(
                            crect,
                            0.0,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(88, 255, 88, 175)),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }
    }

    // 6. Subtle Chunk / Region Grid + (0,0) Origin Axes
    if state.map_layers.grid_chunks {
        // 512-block Region (.mca) lines always subtle
        let rx0 = (min_bx / 512.0).floor() as i32;
        let rx1 = (max_bx / 512.0).ceil() as i32;
        let rz0 = (min_bz / 512.0).floor() as i32;
        let rz1 = (max_bz / 512.0).ceil() as i32;
        if (rx1 - rx0) <= 32 {
            for rx in rx0..=rx1 {
                let x = world_to_screen((rx * 512) as f64, 0.0).x;
                painter.line_segment(
                    [Pos2::new(x, view_rect.top()), Pos2::new(x, view_rect.bottom())],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
            }
            for rz in rz0..=rz1 {
                let y = world_to_screen(0.0, (rz * 512) as f64).y;
                painter.line_segment(
                    [Pos2::new(view_rect.left(), y), Pos2::new(view_rect.right(), y)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 28)),
                );
            }
        }

        // 16-block chunk grid when zoomed in close
        if bpp <= 1.8 {
            let gx0 = (min_bx / 16.0).floor() as i32;
            let gx1 = (max_bx / 16.0).ceil() as i32;
            let gz0 = (min_bz / 16.0).floor() as i32;
            let gz1 = (max_bz / 16.0).ceil() as i32;
            for gx in gx0..=gx1 {
                let x = world_to_screen((gx * 16) as f64, 0.0).x;
                painter.line_segment(
                    [Pos2::new(x, view_rect.top()), Pos2::new(x, view_rect.bottom())],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
            }
            for gz in gz0..=gz1 {
                let y = world_to_screen(0.0, (gz * 16) as f64).y;
                painter.line_segment(
                    [Pos2::new(view_rect.left(), y), Pos2::new(view_rect.right(), y)],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 16)),
                );
            }
        }
    }

    // Center crosshairs at X=0, Z=0
    let origin = world_to_screen(0.0, 0.0);
    if origin.x >= view_rect.left() && origin.x <= view_rect.right() {
        painter.line_segment(
            [Pos2::new(origin.x, view_rect.top()), Pos2::new(origin.x, view_rect.bottom())],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
        );
    }
    if origin.y >= view_rect.top() && origin.y <= view_rect.bottom() {
        painter.line_segment(
            [Pos2::new(view_rect.left(), origin.y), Pos2::new(view_rect.right(), origin.y)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
        );
    }

    // 7. Structure Markers with Screen-Space Clustering + Crisp PNG Sprites
    let zoom_level = 1.0 / bpp.max(0.1);
    let scan = engine.structures_in_view(
        state.map_dim,
        min_bx.floor() as i32,
        min_bz.floor() as i32,
        max_bx.ceil() as i32,
        max_bz.ceil() as i32,
        state.map_layers,
    );

    let mut all_markers: Vec<StructureMarker> = scan.to_vec();
    if state.map_dim == Dimension::Overworld {
        let sp = engine.spawn_point();
        all_markers.push(StructureMarker {
            kind: StructureKind::SpawnPoint,
            x: sp.0,
            z: sp.1,
            variant: 0,
            biome_id: 0,
        });
    }

    // Sort highest priority first so clustering keeps rare/vital structures over common villages
    all_markers.sort_by_key(|m| structure_priority(m.kind));

    let mut placed_screen_pts: Vec<Pos2> = Vec::new();
    let mut hovered_marker: Option<(StructureMarker, String)> = None;

    let active_ver = rcx.versions.resolve_valid_id(&state.prefs.selected_version);
    for m in all_markers {
        if !rcx.versions.is_active(
            crate::data::version_rules::structure_since_version(m.kind),
            None,
            active_ver,
        ) {
            continue;
        }
        if zoom_level < m.kind.min_zoom_visible() {
            continue;
        }
        // Hide inner 96-block End pillar gateway ring unless zoomed in close so (0,0) stays clear
        if m.kind == StructureKind::EndGateway && m.variant >= 1 && m.variant <= 20 && bpp > 2.2 {
            continue;
        }
        let sp = world_to_screen(m.x as f64, m.z as f64);
        if !view_rect.expand(18.0).contains(sp) {
            continue;
        }
        // Keep rare landmark icons visible even if a denser structure sits nearby; only cull dense overlaps
        let cull_dist_sq = match m.kind {
            StructureKind::SpawnPoint
            | StructureKind::Stronghold
            | StructureKind::Mansion
            | StructureKind::EndShip => 0.0,
            StructureKind::AncientCity | StructureKind::Monument => 12.0 * 12.0,
            _ => 28.0 * 28.0,
        };
        if cull_dist_sq > 0.0
            && placed_screen_pts
                .iter()
                .any(|p| p.distance_sq(sp) < cull_dist_sq)
        {
            continue;
        }
        placed_screen_pts.push(sp);

        let badge_r = if matches!(
            m.kind,
            StructureKind::SpawnPoint | StructureKind::Stronghold | StructureKind::EndShip
        ) {
            13.0
        } else {
            11.0
        };
        let accent = m.kind.accent_color();
        painter.circle_filled(sp, badge_r, Color32::from_rgba_unmultiplied(12, 14, 20, 224));
        painter.circle_stroke(sp, badge_r, Stroke::new(1.8_f32, accent));
        let icon_rect = Rect::from_center_size(sp, Vec2::splat(badge_r * 1.45));
        atlas.paint(&painter, icon_rect, m.kind.icon_id(), Color32::WHITE);

        if let Some(ptr) = pointer_pos {
            if ptr.distance(sp) <= badge_r + 4.0 && hovered_marker.is_none() {
                hovered_marker = Some((m.clone(), m.kind.label(lang).to_string()));
            }
        }
    }

    // 8. User Seed Waypoints Overlay
    let mut hovered_waypoint: Option<crate::state::Waypoint> = None;
    if state.map_layers.waypoints {
        for wp in &state.prefs.waypoints {
            if wp.seed != engine.seed || wp.dim != state.map_dim as i32 {
                continue;
            }
            let sp = world_to_screen(wp.x as f64, wp.z as f64);
            if view_rect.expand(24.0).contains(sp) {
                painter.circle_filled(sp, 12.0, Color32::from_rgba_unmultiplied(16, 22, 34, 235));
                painter.circle_stroke(sp, 12.0, Stroke::new(2.0_f32, palette.accent_primary));
                atlas.paint(
                    &painter,
                    Rect::from_center_size(sp, Vec2::splat(16.0)),
                    &wp.icon,
                    Color32::WHITE,
                );
                painter.text(
                    sp + Vec2::new(0.0, 15.0),
                    egui::Align2::CENTER_TOP,
                    &wp.name,
                    egui::FontId::proportional(11.0),
                    Color32::WHITE,
                );
                if let Some(ptr) = pointer_pos {
                    if ptr.distance(sp) <= 16.0 && hovered_waypoint.is_none() {
                        hovered_waypoint = Some(wp.clone());
                    }
                }
            }
        }
    }

    // 8c. Saved Player Location Marker (with uploaded skin face or classic Steve)
    let player_marker_data: Option<(String, [f64; 3], Dimension, String)> = if state.map_layers.player_marker {
        state.active_world_for_seed(engine.seed).and_then(|w| {
            w.player_pos.map(|pos| {
                (
                    w.level_name.clone(),
                    pos,
                    w.player_dimension(),
                    w.last_played_label(lang),
                )
            })
        })
    } else {
        None
    };
    let face_tex_id = state.ensure_player_face_tex(ui.ctx()).id();
    let mut hovered_player: Option<(String, [f64; 3], Dimension, String)> = None;
    if let Some((ref wname, pos, pdim, ref last_lbl)) = player_marker_data {
        if pdim == state.map_dim {
            let sp = world_to_screen(pos[0], pos[2]);
            if view_rect.expand(48.0).contains(sp) {
                let face_size = 24.0;
                let head_rect = Rect::from_center_size(sp, Vec2::splat(face_size));
                // Crisp contrasting ring + pixel-art player face square
                painter.rect_filled(
                    head_rect.expand(3.0),
                    CornerRadius::same(5),
                    Color32::from_rgba_unmultiplied(10, 14, 22, 235),
                );
                painter.rect_stroke(
                    head_rect.expand(2.5),
                    CornerRadius::same(5),
                    Stroke::new(2.0_f32, palette.accent_gold),
                    egui::StrokeKind::Outside,
                );
                painter.image(
                    face_tex_id,
                    head_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Explicit non-live disclosure badge beneath player marker
                let badge_txt = match lang {
                    Language::EnUs => "Last saved (not live)",
                    _ => "Última guardada (no en vivo)",
                };
                let badge_w = (badge_txt.len() as f32 * 5.6 + 12.0).max(122.0);
                let badge_rect = Rect::from_center_size(
                    sp + Vec2::new(0.0, face_size * 0.5 + 12.0),
                    Vec2::new(badge_w, 16.0),
                );
                painter.rect_filled(
                    badge_rect,
                    CornerRadius::same(4),
                    Color32::from_rgba_unmultiplied(12, 16, 26, 226),
                );
                painter.rect_stroke(
                    badge_rect,
                    CornerRadius::same(4),
                    Stroke::new(1.0_f32, palette.accent_gold),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_txt,
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(255, 224, 130),
                );

                if let Some(ptr) = pointer_pos {
                    if (ptr.distance(sp) <= 18.0 || badge_rect.expand(4.0).contains(ptr))
                        && hovered_marker.is_none()
                        && hovered_waypoint.is_none()
                    {
                        hovered_player = Some((wname.clone(), pos, pdim, last_lbl.clone()));
                    }
                }
            }
        }
    }

    // 8b. Temporary Google-Maps-Style Pin Overlay
    let mut temp_pin_screen: Option<Pos2> = None;
    if let Some((pdim, px, pz)) = state.temp_map_pin {
        if pdim == state.map_dim {
            let sp = world_to_screen(px as f64, pz as f64);
            if view_rect.expand(36.0).contains(sp) {
                temp_pin_screen = Some(sp);
                let head = sp - Vec2::new(0.0, 22.0);
                // Ground drop shadow
                painter.circle_filled(
                    sp + Vec2::new(0.0, 1.0),
                    5.5,
                    Color32::from_black_alpha(120),
                );
                // Pointer stem triangle + pin body
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        sp,
                        head + Vec2::new(-8.5, 4.5),
                        head + Vec2::new(8.5, 4.5),
                    ],
                    Color32::from_rgb(225, 58, 52),
                    Stroke::new(1.5_f32, Color32::WHITE),
                ));
                painter.circle_filled(head, 10.5, Color32::from_rgb(232, 62, 54));
                painter.circle_stroke(head, 10.5, Stroke::new(1.8_f32, Color32::WHITE));
                painter.circle_filled(head, 4.0, Color32::from_rgb(36, 14, 16));

                let pin_txt = format!("{}, {}", px, pz);
                let tag_rect = Rect::from_center_size(
                    sp + Vec2::new(0.0, 14.0),
                    Vec2::new((pin_txt.len() as f32 * 6.8 + 12.0).max(58.0), 17.0),
                );
                painter.rect_filled(tag_rect, 4.0, Color32::from_rgba_unmultiplied(16, 18, 26, 220));
                painter.rect_stroke(
                    tag_rect,
                    4.0,
                    Stroke::new(1.0_f32, Color32::from_rgb(232, 62, 54)),
                    egui::StrokeKind::Inside,
                );
                painter.text(
                    tag_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    pin_txt,
                    egui::FontId::monospace(10.5),
                    Color32::WHITE,
                );
            }
        }
    }

    // Prevent clicks on floating map overlays (temp-pin bar or right-click menu) from hitting canvas
    let overlay_rects: (Option<Rect>, Option<Rect>) = ui.ctx().data(|d| {
        (
            d.get_temp::<Rect>(egui::Id::new("seed_map_pin_bar_rect")),
            d.get_temp::<Rect>(egui::Id::new("seed_map_ctx_menu_rect")),
        )
    });
    let pointer_on_overlay = ui.ctx().pointer_latest_pos().is_some_and(|p| {
        overlay_rects.0.is_some_and(|r| r.contains(p))
            || overlay_rects.1.is_some_and(|r| r.contains(p))
    });

    // 9. Interactive Clicks: Ruler | Structure /tp Copy | Temp Google-Maps Pin | Right-Click Menu
    let shift_held = ui.ctx().input(|i| i.modifiers.shift);
    let left_clicked = resp.clicked_by(egui::PointerButton::Primary) && !pointer_on_overlay;
    let right_clicked = resp.clicked_by(egui::PointerButton::Secondary) && !pointer_on_overlay;

    if left_clicked {
        if state.map_context_menu.is_some() {
            state.map_context_menu = None;
        } else if let Some(ptr) = resp.interact_pointer_pos() {
            if state.ruler_active || shift_held {
                let (wx, wz) = screen_to_world(ptr);
                let pt = (wx.round() as i32, wz.round() as i32);
                if state.ruler_p1.is_none() || state.ruler_p2.is_some() {
                    state.ruler_p1 = Some(pt);
                    state.ruler_p2 = None;
                } else {
                    state.ruler_p2 = Some(pt);
                }
                audio.play(SoundCue::Click);
            } else if let Some((ref m, ref title)) = hovered_marker {
                let cmd = format_tp_command(state.map_dim, m.x, m.z);
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", title, cmd),
                    m.kind.icon_id(),
                    m.kind.accent_color(),
                );
            } else if let Some(ref wp) = hovered_waypoint {
                let cmd = format_tp_command(wp.dimension(), wp.x, wp.z);
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", wp.name, cmd),
                    &wp.icon,
                    palette.accent_primary,
                );
            } else if let Some((ref wname, pos, pdim, _)) = hovered_player {
                let cmd = format_tp_xyz(
                    pdim,
                    pos[0].round() as i32,
                    Some(pos[1].round() as i32),
                    pos[2].round() as i32,
                );
                ui.ctx().copy_text(cmd.clone());
                audio.play(SoundCue::Toast);
                toasts.push(
                    t(lang, UiKey::CopiedTpTitle),
                    format!("{}: {}", wname, cmd),
                    "compass",
                    palette.accent_gold,
                );
            } else {
                let clicked_existing_pin = temp_pin_screen.is_some_and(|ps| {
                    ptr.distance(ps) <= 22.0 || ptr.distance(ps - Vec2::new(0.0, 18.0)) <= 16.0
                });
                if clicked_existing_pin {
                    state.temp_map_pin = None;
                } else {
                    let (wx, wz) = screen_to_world(ptr);
                    state.temp_map_pin =
                        Some((state.map_dim, wx.round() as i32, wz.round() as i32));
                }
                audio.play(SoundCue::Click);
            }
        }
    }

    if right_clicked {
        if let Some(ptr) = resp.interact_pointer_pos() {
            let (bx, bz, suggested) = if let Some((ref m, ref title)) = hovered_marker {
                (m.x, m.z, title.clone())
            } else {
                let (wx, wz) = screen_to_world(ptr);
                (wx.round() as i32, wz.round() as i32, String::new())
            };
            state.map_context_menu = Some(MapContextMenuState {
                dim: state.map_dim,
                x: bx,
                z: bz,
                screen_pos: ptr,
                name: suggested,
                icon: state.new_waypoint_icon.clone(),
                focus_name_next: true,
            });
            audio.play(SoundCue::Click);
        }
    }
    if let Some(p1) = state.ruler_p1 {
        let s1 = world_to_screen(p1.0 as f64, p1.1 as f64);
        painter.circle_filled(s1, 5.0, palette.accent_gold);
        let p2_world = state.ruler_p2.or_else(|| {
            pointer_pos.map(|p| {
                let (wx, wz) = screen_to_world(p);
                (wx.round() as i32, wz.round() as i32)
            })
        });
        if let Some(p2) = p2_world {
            let s2 = world_to_screen(p2.0 as f64, p2.1 as f64);
            painter.line_segment([s1, s2], Stroke::new(2.0_f32, palette.accent_gold));
            painter.circle_filled(s2, 5.0, palette.accent_gold);
            let dx = (p2.0 - p1.0) as f64;
            let dz = (p2.1 - p1.1) as f64;
            let dist = (dx * dx + dz * dz).sqrt().round() as i64;
            let mid = Pos2::new((s1.x + s2.x) * 0.5, (s1.y + s2.y) * 0.5);
            painter.text(
                mid + Vec2::new(0.0, -10.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{} blocks (Nether: {})", dist, (dist as f64 / 8.0).round() as i64),
                egui::FontId::monospace(12.0),
                palette.accent_gold,
            );
        }
    }

    // Floating Temp-Pin Quick Bar inside top-right corner of map
    let mut new_pin_bar_rect: Option<Rect> = None;
    if let Some((pdim, px, pz)) = state.temp_map_pin {
        if pdim == state.map_dim {
            let area_resp = egui::Area::new(egui::Id::new("seed_map_temp_pin_bar"))
                .order(egui::Order::Middle)
                .fixed_pos(Pos2::new(
                    (view_rect.right() - 370.0).max(view_rect.left() + 8.0),
                    view_rect.top() + 8.0,
                ))
                .constrain_to(view_rect)
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(
                            palette.bg_panel.r(),
                            palette.bg_panel.g(),
                            palette.bg_panel.b(),
                            235,
                        ))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(232, 62, 54)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(atlas.ui_image("compass", 15.0));
                                ui.label(
                                    RichText::new(format!(
                                        "{}: {}, {}",
                                        t(lang, UiKey::TempPinLabel),
                                        px,
                                        pz
                                    ))
                                    .monospace()
                                    .strong()
                                    .size(11.5)
                                    .color(palette.text_primary),
                                );
                                if ui.small_button(t(lang, UiKey::CopyTpBtn)).clicked() {
                                    let cmd = format_tp_command(pdim, px, pz);
                                    ui.ctx().copy_text(cmd.clone());
                                    audio.play(SoundCue::Toast);
                                    toasts.push(
                                        t(lang, UiKey::CopiedTpTitle),
                                        cmd,
                                        "compass",
                                        palette.accent_primary,
                                    );
                                }
                                if ui.small_button(t(lang, UiKey::SaveWaypointHereBtn)).clicked() {
                                    let sp = world_to_screen(px as f64, pz as f64);
                                    state.map_context_menu = Some(MapContextMenuState {
                                        dim: pdim,
                                        x: px,
                                        z: pz,
                                        screen_pos: sp,
                                        name: String::new(),
                                        icon: state.new_waypoint_icon.clone(),
                                        focus_name_next: true,
                                    });
                                    audio.play(SoundCue::Click);
                                }
                                if ui
                                    .small_button("✕")
                                    .on_hover_text(t(lang, UiKey::RemovePinBtn))
                                    .clicked()
                                {
                                    state.temp_map_pin = None;
                                    audio.play(SoundCue::Click);
                                }
                            });
                        });
                });
            new_pin_bar_rect = Some(area_resp.response.rect);
        }
    }

    // Floating Right-Click Context Menu for Custom Waypoints (12 Icons + Exact Coords)
    let mut new_ctx_menu_rect: Option<Rect> = None;
    let mut close_ctx_menu = false;
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        close_ctx_menu = true;
    }
    if let Some(mut menu) = state.map_context_menu.take() {
        let area_resp = egui::Area::new(egui::Id::new("seed_map_ctx_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu.screen_pos + Vec2::new(8.0, 8.0))
            .constrain_to(ui.ctx().screen_rect().shrink(8.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(15, 19, 28, 246))
                    .stroke(Stroke::new(1.5_f32, palette.accent_primary))
                    .corner_radius(CornerRadius::same(7))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_width(250.0);
                        ui.horizontal(|ui| {
                            ui.add(atlas.ui_image(&menu.icon, 18.0));
                            ui.label(
                                RichText::new(t(lang, UiKey::RightClickMenuTitle))
                                    .strong()
                                    .size(12.5)
                                    .color(palette.accent_gold),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").clicked() {
                                    close_ctx_menu = true;
                                }
                            });
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} • X: {}  Z: {}",
                                menu.dim.label(lang),
                                menu.x,
                                menu.z
                            ))
                            .monospace()
                            .size(11.0)
                            .color(palette.text_muted),
                        );
                        ui.add_space(6.0);

                        let te = ui.add_sized(
                            [230.0, 24.0],
                            egui::TextEdit::singleline(&mut menu.name)
                                .hint_text(t(lang, UiKey::WaypointNameHint)),
                        );
                        if menu.focus_name_next {
                            te.request_focus();
                            menu.focus_name_next = false;
                        }
                        let submit_enter = te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("{}:", t(lang, UiKey::WaypointIconLabel)))
                                .size(11.0)
                                .color(palette.text_muted),
                        );
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                            for (icon_id, es_lbl, en_lbl) in WAYPOINT_ICON_PRESETS {
                                let active = menu.icon == icon_id;
                                let (sr, sresp) =
                                    ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                                paint_mc_slot(
                                    ui.painter(),
                                    sr,
                                    palette,
                                    atlas,
                                    Some(icon_id),
                                    SlotStyle {
                                        selected: active,
                                        hovered: sresp.hovered(),
                                        ..Default::default()
                                    },
                                );
                                let tip = if matches!(lang, Language::EnUs) { en_lbl } else { es_lbl };
                                if sresp.on_hover_text(tip).clicked() {
                                    menu.icon = icon_id.to_string();
                                    audio.play(SoundCue::Click);
                                }
                            }
                        });

                        ui.add_space(8.0);
                        let save_clicked = ui
                            .add_sized(
                                [230.0, 26.0],
                                egui::Button::new(
                                    RichText::new(t(lang, UiKey::SaveWaypointHereBtn))
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgba_unmultiplied(
                                    palette.accent_xp.r(),
                                    palette.accent_xp.g(),
                                    palette.accent_xp.b(),
                                    65,
                                ))
                                .stroke(Stroke::new(1.5_f32, palette.accent_xp)),
                            )
                            .clicked();

                        if save_clicked || submit_enter {
                            let label_shown = if menu.name.trim().is_empty() {
                                format!("({}, {})", menu.x, menu.z)
                            } else {
                                menu.name.trim().to_string()
                            };
                            state.new_waypoint_icon = menu.icon.clone();
                            state.add_waypoint_at(
                                engine.seed,
                                menu.dim,
                                menu.x,
                                menu.z,
                                &label_shown,
                                &menu.icon,
                            );
                            audio.play(SoundCue::XpOrb);
                            toasts.push(
                                t(lang, UiKey::ToastWaypointAddedTitle),
                                label_shown,
                                &menu.icon,
                                palette.accent_primary,
                            );
                            close_ctx_menu = true;
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button(t(lang, UiKey::CopyTpBtn)).clicked() {
                                let cmd = format_tp_command(menu.dim, menu.x, menu.z);
                                ui.ctx().copy_text(cmd.clone());
                                audio.play(SoundCue::Toast);
                                toasts.push(
                                    t(lang, UiKey::CopiedTpTitle),
                                    cmd,
                                    "compass",
                                    palette.accent_primary,
                                );
                                close_ctx_menu = true;
                            }
                            if ui.button(t(lang, UiKey::TempPinLabel)).clicked() {
                                state.temp_map_pin = Some((menu.dim, menu.x, menu.z));
                                audio.play(SoundCue::Click);
                                close_ctx_menu = true;
                            }
                        });
                    });
            });
        new_ctx_menu_rect = Some(area_resp.response.rect);
        if !close_ctx_menu {
            state.map_context_menu = Some(menu);
        }
    }

    ui.ctx().data_mut(|d| {
        if let Some(r) = new_pin_bar_rect {
            d.insert_temp(egui::Id::new("seed_map_pin_bar_rect"), r);
        } else {
            d.remove::<Rect>(egui::Id::new("seed_map_pin_bar_rect"));
        }
        if let Some(r) = new_ctx_menu_rect {
            d.insert_temp(egui::Id::new("seed_map_ctx_menu_rect"), r);
        } else {
            d.remove::<Rect>(egui::Id::new("seed_map_ctx_menu_rect"));
        }
    });

    // Tooltip for hovered structure badge or waypoint (only when menu isn't covering pointer)
    if !pointer_on_overlay {
        if let (Some(ptr), Some((m, title))) = (pointer_pos, hovered_marker) {
            let coords = format!("X: {}  Z: {}", m.x, m.z);
            let nether_link = format!("Nether (÷8): {}, {}", m.x.div_euclid(8), m.z.div_euclid(8));
            let tp_cmd = format_tp_command(state.map_dim, m.x, m.z);
            let tp_hint = match lang {
                Language::EnUs => format!("Left-Click -> {}", tp_cmd),
                _ => format!("Clic izq. -> {}", tp_cmd),
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &title,
                &[
                    (&coords, palette.accent_primary),
                    (&nether_link, palette.text_muted),
                    (&tp_hint, palette.accent_gold),
                ],
                palette,
            );
        } else if let (Some(ptr), Some(wp)) = (pointer_pos, hovered_waypoint) {
            let coords = format!("X: {}  Z: {}", wp.x, wp.z);
            let tp_cmd = format_tp_command(wp.dimension(), wp.x, wp.z);
            let tp_hint = match lang {
                Language::EnUs => format!("Left-Click -> {}", tp_cmd),
                _ => format!("Clic izq. -> {}", tp_cmd),
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &wp.name,
                &[
                    (&coords, palette.accent_primary),
                    (&tp_hint, palette.accent_gold),
                ],
                palette,
            );
        } else if let (Some(ptr), Some((wname, pos, pdim, last_lbl))) = (pointer_pos, hovered_player) {
            let not_live = t(lang, UiKey::PlayerNotLiveBadge);
            let coords = format!(
                "{} • X: {:.0}  Y: {:.0}  Z: {:.0}",
                pdim.label(lang),
                pos[0],
                pos[1],
                pos[2]
            );
            let played = match lang {
                Language::EnUs => format!("Last played: {}", last_lbl),
                _ => format!("Último guardado: {}", last_lbl),
            };
            let tp_hint = match lang {
                Language::EnUs => "Left-Click -> Copy /tp to saved position",
                _ => "Clic izq. -> Copiar /tp a posición guardada",
            };
            show_mc_tooltip_at(
                ui.ctx(),
                ptr,
                &format!("{} - {}", t(lang, UiKey::PlayerMarkerTitle), wname),
                &[
                    (not_live, Color32::from_rgb(255, 210, 110)),
                    (&coords, palette.accent_primary),
                    (&played, palette.text_muted),
                    (tp_hint, palette.accent_gold),
                ],
                palette,
            );
        }
    }

    ui.add_space(6.0);

    // 10. Bottom Live Coordinate / Biome / Chunk / Slime HUD
    let sample_pos = if let Some(ptr) = pointer_pos {
        let (wx, wz) = screen_to_world(ptr);
        (wx.round() as i32, wz.round() as i32)
    } else {
        (state.map_center_x.round() as i32, state.map_center_z.round() as i32)
    };
    state.last_cursor_world = Some(sample_pos);

    let (_bid, approx_y, biome_name) =
        engine.sample_point(state.map_dim, sample_pos.0, sample_pos.1, lang);
    let chunk_x = sample_pos.0.div_euclid(16);
    let chunk_z = sample_pos.1.div_euclid(16);
    let reg_x = sample_pos.0.div_euclid(512);
    let reg_z = sample_pos.1.div_euclid(512);
    let slime =
        state.map_dim == Dimension::Overworld && is_slime_chunk(engine.seed, chunk_x, chunk_z);

    egui::Frame::new()
        .fill(palette.bg_panel)
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(12.0, 4.0);
                ui.label(
                    RichText::new(format!(
                        "{}: {}, {}",
                        t(lang, UiKey::CursorHudBlock),
                        sample_pos.0,
                        sample_pos.1
                    ))
                    .monospace()
                    .strong()
                    .size(12.0)
                    .color(palette.accent_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}, {}",
                        t(lang, UiKey::CursorHudChunk),
                        chunk_x,
                        chunk_z
                    ))
                    .monospace()
                    .size(11.5)
                    .color(palette.text_primary),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: r.{}.{}.mca",
                        t(lang, UiKey::CursorHudRegion),
                        reg_x,
                        reg_z
                    ))
                    .monospace()
                    .size(11.5)
                    .color(palette.text_muted),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}",
                        t(lang, UiKey::CursorHudBiome),
                        biome_name
                    ))
                    .size(12.0)
                    .strong()
                    .color(palette.accent_gold),
                );
                if state.map_dim == Dimension::Overworld {
                    ui.label(
                        RichText::new(format!("{}: ~{}", t(lang, UiKey::CursorHudHeight), approx_y.round() as i32))
                            .size(11.5)
                            .color(palette.text_muted),
                    );
                    ui.label(
                        RichText::new(if slime {
                            t(lang, UiKey::CursorHudSlimeYes)
                        } else {
                            t(lang, UiKey::CursorHudSlimeNo)
                        })
                        .size(11.5)
                        .strong()
                        .color(if slime { palette.accent_xp } else { palette.text_muted }),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}: {}, {}",
                            t(lang, UiKey::LinkedCoordsNether),
                            sample_pos.0.div_euclid(8),
                            sample_pos.1.div_euclid(8)
                        ))
                        .monospace()
                        .size(11.5)
                        .color(Color32::from_rgb(255, 130, 96)),
                    );
                } else if state.map_dim == Dimension::Nether {
                    ui.label(
                        RichText::new(format!(
                            "{}: {}, {}",
                            t(lang, UiKey::LinkedCoordsOverworld),
                            sample_pos.0 * 8,
                            sample_pos.1 * 8
                        ))
                        .monospace()
                        .size(11.5)
                        .color(palette.accent_xp),
                    );
                }
                if engine.has_in_flight() {
                    ui.label(
                        RichText::new(t(lang, UiKey::LoadingTiles))
                            .size(11.0)
                            .color(palette.accent_gold),
                    );
                }
            });
        });
}

pub(super) fn pick_cubiomes_scale(bpp: f32) -> i32 {
    if bpp <= 1.6 {
        1
    } else if bpp <= 6.5 {
        4
    } else if bpp <= 24.0 {
        16
    } else if bpp <= 68.0 {
        64
    } else {
        256
    }
}

fn structure_priority(kind: StructureKind) -> u8 {
    match kind {
        StructureKind::SpawnPoint => 0,
        StructureKind::Stronghold => 1,
        StructureKind::EndShip => 2,
        StructureKind::Mansion => 3,
        StructureKind::AncientCity => 4,
        StructureKind::Monument => 5,
        StructureKind::EndGateway => 6,
        StructureKind::Bastion => 7,
        StructureKind::Fortress => 8,
        StructureKind::EndCity => 9,
        StructureKind::TrialChambers => 10,
        StructureKind::Village => 11,
        StructureKind::Outpost => 12,
        StructureKind::DesertPyramid | StructureKind::JungleTemple => 13,
        _ => 15,
    }
}

