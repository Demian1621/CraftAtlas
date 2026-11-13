use eframe::egui::{self, Align, Color32, CornerRadius, CursorIcon, Layout, Margin, Pos2, Rect, ResizeDirection, RichText, Sense, Stroke, Vec2, ViewportCommand};

use crate::assets::ItemAtlas;
use crate::audio::{AudioEngine, SoundCue};
use crate::data::models::{Database, Language, VersionRegistry};
use crate::data::recipes::RecipeDatabase;
use crate::i18n::{t, UiKey};
use crate::state::{AppState, AppTab, RecipeViewMode, ThemeId};
use crate::ui::theme::{filter_chip, paint_mc_slot, McPalette};

/// Handles frameless window edge/corner resize strips and draws a sharp 1px outer border when restored.
pub fn render_borderless_window_frame(ctx: &egui::Context, palette: McPalette) {
    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let screen = ctx.screen_rect();

    let fg = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("mc_window_outline"),
    ));
    if !is_maximized {
        fg.rect_stroke(
            screen,
            0.0,
            Stroke::new(1.0_f32, palette.border_strong),
            egui::StrokeKind::Inside,
        );
    } else {
        return;
    }

    let edge = 5.0_f32;
    let corner = 9.0_f32;
    let zones: [(&str, Rect, ResizeDirection, CursorIcon); 8] = [
        (
            "nw",
            Rect::from_min_max(screen.min, Pos2::new(screen.left() + corner, screen.top() + corner)),
            ResizeDirection::NorthWest,
            CursorIcon::ResizeNorthWest,
        ),
        (
            "ne",
            Rect::from_min_max(Pos2::new(screen.right() - corner, screen.top()), Pos2::new(screen.right(), screen.top() + corner)),
            ResizeDirection::NorthEast,
            CursorIcon::ResizeNorthEast,
        ),
        (
            "sw",
            Rect::from_min_max(Pos2::new(screen.left(), screen.bottom() - corner), Pos2::new(screen.left() + corner, screen.bottom())),
            ResizeDirection::SouthWest,
            CursorIcon::ResizeSouthWest,
        ),
        (
            "se",
            Rect::from_min_max(Pos2::new(screen.right() - corner, screen.bottom() - corner), screen.max),
            ResizeDirection::SouthEast,
            CursorIcon::ResizeSouthEast,
        ),
        (
            "n",
            Rect::from_min_max(Pos2::new(screen.left() + corner, screen.top()), Pos2::new(screen.right() - corner, screen.top() + edge)),
            ResizeDirection::North,
            CursorIcon::ResizeVertical,
        ),
        (
            "s",
            Rect::from_min_max(Pos2::new(screen.left() + corner, screen.bottom() - edge), Pos2::new(screen.right() - corner, screen.bottom())),
            ResizeDirection::South,
            CursorIcon::ResizeVertical,
        ),
        (
            "w",
            Rect::from_min_max(Pos2::new(screen.left(), screen.top() + corner), Pos2::new(screen.left() + edge, screen.bottom() - corner)),
            ResizeDirection::West,
            CursorIcon::ResizeHorizontal,
        ),
        (
            "e",
            Rect::from_min_max(Pos2::new(screen.right() - edge, screen.top() + corner), Pos2::new(screen.right(), screen.bottom() - corner)),
            ResizeDirection::East,
            CursorIcon::ResizeHorizontal,
        ),
    ];

    egui::Area::new(egui::Id::new("mc_window_resize_handles"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(false)
        .show(ctx, |ui| {
            let primary_pressed = ui.ctx().input(|i| i.pointer.primary_pressed());
            for (tag, rect, dir, cursor) in zones {
                let id = ui.make_persistent_id(("resize", tag));
                let resp = ui.interact(rect, id, Sense::click_and_drag());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(cursor);
                    if primary_pressed || resp.drag_started() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(dir));
                    }
                }
            }
        });
}

/// Renders the integrated borderless title bar (drag, double-click maximize, Min/Max/Close, Version selector)
/// and clean module navigation + action bar.
pub fn render_top_chrome(
    ctx: &egui::Context,
    state: &mut AppState,
    versions: &VersionRegistry,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
) {
    // Ctrl+F shortcut anywhere inside the window toggles/focuses Universal Command Palette
    if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
        state.search_palette_open = true;
        state.focus_search_next_frame = true;
        state.palette_selected_idx = 0;
    }

    let lang = state.prefs.language;
    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();
    if state.prefs.selected_version != active_ver {
        state.prefs.selected_version = active_ver.clone();
    }

    egui::TopBottomPanel::top("mc_top_header")
        .frame(
            egui::Frame::new()
                .fill(palette.bg_panel)
                .inner_margin(Margin::symmetric(12, 6))
                .stroke(Stroke::new(1.0_f32, palette.border_subtle)),
        )
        .show(ctx, |ui| {
            // Row 1: Integrated Window Title Bar (Emblem + App Title + Version Selector + Drag Zone + Min/Max/Close)
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 28.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                    ui.add(atlas.ui_image("totem_of_undying", 22.0));
                    ui.label(
                        RichText::new(t(lang, UiKey::AppTitle))
                            .size(14.5)
                            .strong()
                            .color(palette.text_primary),
                    );

                    // Interactive Minecraft Version Selector (driven by data/versions.json)
                    let active_def = versions.find(&active_ver);
                    let ver_label = active_def
                        .map(|v| v.label.as_str())
                        .unwrap_or(&active_ver);
                    let ver_resp = egui::ComboBox::from_id_salt("mc_version_selector")
                        .selected_text(RichText::new(ver_label).strong().color(palette.accent_xp))
                        .width(108.0)
                        .show_ui(ui, |ui| {
                            for v in &versions.versions {
                                let selected = state.prefs.selected_version == v.id;
                                if ui.selectable_label(selected, &v.label).clicked() {
                                    state.prefs.selected_version = v.id.clone();
                                    state.prefs.save();
                                    audio.play(SoundCue::Click);
                                }
                            }
                        });
                    if let Some(def) = active_def {
                        ver_resp.response.on_hover_ui(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} - {}",
                                    def.label,
                                    def.subtitle.get(lang)
                                ))
                                .strong()
                                .color(palette.accent_xp),
                            );
                            if !def.highlights.is_empty() {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(t(lang, UiKey::VersionChangelogHeader))
                                        .size(11.0)
                                        .color(palette.text_muted),
                                );
                                for h in &def.highlights {
                                    ui.label(RichText::new(format!("• {}", h.get(lang))).size(11.0));
                                }
                            }
                        });
                    }

                    render_app_version_pill(ui, ctx, state, palette, lang, audio);

                    // Middle drag & double-click maximize strip + Right-aligned Min/Max/Close controls
                    let remaining_w = (ui.available_width() - 96.0).max(24.0);
                    let (drag_rect, drag_resp) = ui.allocate_exact_size(
                        Vec2::new(remaining_w, 26.0),
                        Sense::click_and_drag(),
                    );
                    if drag_resp.drag_started() {
                        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    if drag_resp.double_clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
                    }
                    if drag_resp.hovered() {
                        // Subtle center drag grip hint dots
                        let c = drag_rect.center();
                        for dx in [-10.0_f32, 0.0, 10.0] {
                            ui.painter().circle_filled(
                                Pos2::new(c.x + dx, c.y),
                                1.5,
                                Color32::from_rgba_unmultiplied(
                                    palette.text_muted.r(),
                                    palette.text_muted.g(),
                                    palette.text_muted.b(),
                                    110,
                                ),
                            );
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;

                        // Close button
                        if window_chrome_btn(
                            ui,
                            "✕",
                            palette.accent_danger,
                            palette,
                            t(lang, UiKey::WindowClose),
                        )
                        .clicked()
                        {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }

                        // Maximize / Restore button
                        let max_glyph = if is_maximized { "❐" } else { "□" };
                        if window_chrome_btn(
                            ui,
                            max_glyph,
                            palette.accent_primary,
                            palette,
                            t(lang, UiKey::WindowMaximize),
                        )
                        .clicked()
                        {
                            ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
                        }

                        // Minimize button
                        if window_chrome_btn(
                            ui,
                            "—",
                            palette.accent_gold,
                            palette,
                            t(lang, UiKey::WindowMinimize),
                        )
                        .clicked()
                        {
                            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                        }
                    });
                },
            );

            ui.add_space(4.0);
            ui.painter().line_segment(
                [
                    Pos2::new(ui.max_rect().left(), ui.cursor().top()),
                    Pos2::new(ui.max_rect().right(), ui.cursor().top()),
                ],
                Stroke::new(1.0_f32, palette.border_subtle),
            );
            ui.add_space(5.0);

            // Row 2: 6 Module Tabs + Spacious Utility Controls (Search Palette trigger, Favs, Audio, Themes, Lang)
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                for tab in AppTab::all() {
                    let lbl = match tab {
                        AppTab::Gear => t(lang, UiKey::TabGear),
                        AppTab::Recipes => t(lang, UiKey::TabRecipes),
                        AppTab::SeedMap => t(lang, UiKey::TabSeedMap),
                        AppTab::Farms => t(lang, UiKey::TabFarms),
                        AppTab::Dupes => t(lang, UiKey::TabDupes),
                        AppTab::ProTips => t(lang, UiKey::TabProTips),
                    };
                    if filter_chip(
                        ui,
                        palette,
                        atlas,
                        Some(tab.icon_id()),
                        lbl,
                        state.active_tab == tab,
                    )
                    .clicked()
                    {
                        state.switch_tab(tab);
                        audio.play(SoundCue::Click);
                    }
                }

                ui.separator();

                // Clean floating Command Palette trigger (replaces cramped top Search input)
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("spyglass"),
                    t(lang, UiKey::SearchTrigger),
                    state.search_palette_open,
                )
                .clicked()
                {
                    state.search_palette_open = !state.search_palette_open;
                    state.focus_search_next_frame = state.search_palette_open;
                    state.palette_selected_idx = 0;
                    audio.play(SoundCue::Click);
                }

                // Favorites rail toggle
                let fav_active = state.prefs.favorites_open;
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some("nether_star"),
                    &format!(
                        "{} ({})",
                        t(lang, UiKey::FavoritesToggle),
                        state.prefs.favorite_items.len()
                            + state.prefs.favorite_gear.len()
                            + state.prefs.favorite_farms.len()
                    ),
                    fav_active,
                )
                .clicked()
                {
                    state.prefs.favorites_open = !state.prefs.favorites_open;
                    state.prefs.save();
                    audio.play(SoundCue::Click);
                }

                // Audio mute/unmute toggle
                let sound_lbl = if state.prefs.audio_enabled {
                    t(lang, UiKey::AudioOn)
                } else {
                    t(lang, UiKey::AudioOff)
                };
                if filter_chip(
                    ui,
                    palette,
                    atlas,
                    Some(if state.prefs.audio_enabled { "note_block" } else { "barrier" }),
                    sound_lbl,
                    state.prefs.audio_enabled,
                )
                .clicked()
                {
                    state.prefs.audio_enabled = !state.prefs.audio_enabled;
                    state.prefs.save();
                }

                // Theme switcher (6 scenic themes)
                let cur_theme = state.prefs.theme;
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(cur_theme.label(lang))
                    .width(146.0)
                    .show_ui(ui, |ui| {
                        for th in ThemeId::all() {
                            if ui
                                .selectable_value(&mut state.prefs.theme, th, th.label(lang))
                                .changed()
                            {
                                state.prefs.save();
                                audio.play(SoundCue::Click);
                            }
                        }
                    });

                // Language switcher
                let cur_lang = state.prefs.language;
                egui::ComboBox::from_id_salt("lang_selector")
                    .selected_text(cur_lang.short_label())
                    .width(74.0)
                    .show_ui(ui, |ui| {
                        for l in Language::all() {
                            if ui
                                .selectable_value(&mut state.prefs.language, l, l.display_name())
                                .changed()
                            {
                                state.prefs.save();
                                audio.play(SoundCue::Click);
                            }
                        }
                    });
            });
        });
}

fn window_chrome_btn(
    ui: &mut egui::Ui,
    glyph: &str,
    hover_accent: Color32,
    palette: McPalette,
    tooltip: &str,
) -> egui::Response {
    let size = Vec2::new(26.0, 22.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let bg = if resp.hovered() {
        Color32::from_rgba_unmultiplied(hover_accent.r(), hover_accent.g(), hover_accent.b(), 190)
    } else {
        Color32::from_rgba_unmultiplied(
            palette.bg_card.r(),
            palette.bg_card.g(),
            palette.bg_card.b(),
            175,
        )
    };
    let fg = if resp.hovered() {
        Color32::WHITE
    } else {
        palette.text_primary
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(4),
        bg,
        Stroke::new(
            1.0_f32,
            if resp.hovered() {
                hover_accent
            } else {
                palette.border_subtle
            },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(13.0),
        fg,
    );
    resp.on_hover_text(tooltip)
}

fn render_app_version_pill(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut AppState,
    palette: McPalette,
    lang: crate::data::models::Language,
    audio: &AudioEngine,
) {
    use crate::updater::{UpdateStatus, CURRENT_VERSION};

    let available_info = match state.updater.status() {
        UpdateStatus::Available(info) => Some(info.clone()),
        _ => None,
    };

    if let Some(info) = available_info {
        let label = format!("{} v{}", t(lang, UiKey::UpdateAvailableBadge), info.version);
        let btn = egui::Button::new(
            RichText::new(label)
                .size(11.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(palette.accent_xp.gamma_multiply(0.85))
        .stroke(Stroke::new(1.0_f32, palette.accent_xp))
        .corner_radius(CornerRadius::same(5));
        let resp = ui.add(btn);
        let url = info.html_url.clone();
        let tooltip = if info.notes_summary.is_empty() {
            format!("v{} -> v{}\n{}", CURRENT_VERSION, info.version, t(lang, UiKey::UpdateAvailableHint))
        } else {
            format!(
                "{} (v{} -> v{})\n\n{}",
                t(lang, UiKey::UpdateAvailableHint),
                CURRENT_VERSION,
                info.version,
                info.notes_summary
            )
        };
        if resp.on_hover_text(tooltip).clicked() {
            audio.play(SoundCue::Click);
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        return;
    }

    let (text, text_color, tooltip) = match state.updater.status() {
        UpdateStatus::Checking => (
            format!("v{} ...", CURRENT_VERSION),
            palette.text_muted,
            t(lang, UiKey::UpdateCheckingTooltip),
        ),
        UpdateStatus::UpToDate => (
            format!("v{}", CURRENT_VERSION),
            palette.text_muted,
            t(lang, UiKey::UpdateUpToDateTooltip),
        ),
        UpdateStatus::Unavailable => (
            format!("v{}", CURRENT_VERSION),
            palette.text_muted,
            t(lang, UiKey::UpdateUnavailableTooltip),
        ),
        UpdateStatus::Available(_) => unreachable!(),
    };

    let btn = egui::Button::new(RichText::new(text).size(10.5).color(text_color))
        .fill(Color32::from_rgba_unmultiplied(
            palette.bg_card.r(),
            palette.bg_card.g(),
            palette.bg_card.b(),
            145,
        ))
        .stroke(Stroke::new(1.0_f32, palette.border_subtle))
        .corner_radius(CornerRadius::same(5));
    let resp = ui.add(btn);
    if resp.on_hover_text(tooltip).clicked() {
        audio.play(SoundCue::Click);
        state.updater.trigger_check();
    }
}

#[derive(Clone)]
enum PaletteTarget {
    Gear(String),
    Recipe(String),
    Farm(String),
    Dupe(String),
    Tip(String),
}

struct PaletteHit {
    icon: String,
    title: String,
    module_label: &'static str,
    accent: Color32,
    target: PaletteTarget,
}

/// Centered floating Universal Command Palette (opened via Ctrl+F or top Search button).
pub fn render_search_palette(
    ctx: &egui::Context,
    state: &mut AppState,
    versions: &VersionRegistry,
    db: &Database,
    recipes: &RecipeDatabase,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
) {
    if !state.search_palette_open {
        return;
    }
    let lang = state.prefs.language;
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.search_palette_open = false;
        return;
    }

    let screen = ctx.screen_rect();
    // Dim click-catcher backdrop
    egui::Area::new(egui::Id::new("mc_cmd_palette_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (_r, bg_resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(150));
            if bg_resp.clicked() {
                state.search_palette_open = false;
            }
        });

    let q = state.palette_query.trim().to_lowercase();
    let mut hits: Vec<PaletteHit> = Vec::with_capacity(40);

    // Collect top version-active matches across Gear, JEI Items, Farms, Dupes, Pro Tips
    for g in &db.gear {
        if !versions.is_active(g.since_version.as_deref(), g.until_version.as_deref(), &active_ver) {
            continue;
        }
        let res = g.resolved(&active_ver);
        if q.is_empty()
            || res.name.matches(&q)
            || res.what_it_does.matches(&q)
            || g.id.to_lowercase().contains(&q)
        {
            hits.push(PaletteHit {
                icon: g.icon.clone(),
                title: res.name.get(lang).to_string(),
                module_label: t(lang, UiKey::TabGear),
                accent: palette.accent_primary,
                target: PaletteTarget::Gear(g.id.clone()),
            });
        }
        if hits.len() >= 10 && q.is_empty() {
            break;
        }
    }

    let mut recipe_added = 0usize;
    for item_id in &recipes.catalog_items {
        if !recipes.item_active_in(item_id, versions, &active_ver) {
            continue;
        }
        let name = recipes.item_name(item_id, lang);
        let en_name = recipes.item_name(item_id, Language::EnUs);
        let matched = if q.is_empty() {
            matches!(
                item_id.as_str(),
                "crafter" | "mace" | "elytra" | "beacon" | "shulker_box" | "heavy_core"
            )
        } else {
            name.to_lowercase().contains(&q)
                || en_name.to_lowercase().contains(&q)
                || item_id.to_lowercase().contains(&q)
        };
        if matched {
            hits.push(PaletteHit {
                icon: item_id.clone(),
                title: name,
                module_label: t(lang, UiKey::TabRecipes),
                accent: palette.accent_gold,
                target: PaletteTarget::Recipe(item_id.clone()),
            });
            recipe_added += 1;
            if recipe_added >= 14 {
                break;
            }
        }
    }

    for f in &db.farms {
        if !versions.is_active(f.since_version.as_deref(), f.until_version.as_deref(), &active_ver) {
            continue;
        }
        let res = f.resolved(&active_ver);
        if q.is_empty()
            || res.title.matches(&q)
            || res.desc.matches(&q)
            || f.id.to_lowercase().contains(&q)
        {
            let icon = res.resolved_icon().to_string();
            hits.push(PaletteHit {
                icon,
                title: res.title.get(lang).to_string(),
                module_label: t(lang, UiKey::TabFarms),
                accent: palette.accent_xp,
                target: PaletteTarget::Farm(f.id.clone()),
            });
        }
        if q.is_empty() && hits.len() >= 22 {
            break;
        }
    }

    for d in &db.dupes {
        if !versions.is_active(d.since_version.as_deref(), d.until_version.as_deref(), &active_ver) {
            continue;
        }
        let res = d.resolved(&active_ver);
        if (!q.is_empty() && (res.title.matches(&q) || res.how_it_works.matches(&q) || d.id.to_lowercase().contains(&q)))
            || (q.is_empty() && hits.len() < 26)
        {
            hits.push(PaletteHit {
                icon: res.icon.clone(),
                title: res.title.get(lang).to_string(),
                module_label: t(lang, UiKey::TabDupes),
                accent: palette.accent_danger,
                target: PaletteTarget::Dupe(d.id.clone()),
            });
        }
        if hits.len() >= 38 {
            break;
        }
    }

    for tip in &db.pro_tips {
        if !versions.is_active(tip.since_version.as_deref(), tip.until_version.as_deref(), &active_ver) {
            continue;
        }
        let res = tip.resolved(&active_ver);
        if (!q.is_empty() && (res.title.matches(&q) || res.text.matches(&q) || tip.id.to_lowercase().contains(&q)))
            || (q.is_empty() && hits.len() < 30)
        {
            hits.push(PaletteHit {
                icon: res.icon.clone(),
                title: res.title.get(lang).to_string(),
                module_label: t(lang, UiKey::TabProTips),
                accent: palette.accent_enchant,
                target: PaletteTarget::Tip(tip.id.clone()),
            });
        }
        if hits.len() >= 48 {
            break;
        }
    }

    if !hits.is_empty() {
        state.palette_selected_idx = state.palette_selected_idx.min(hits.len() - 1);
    } else {
        state.palette_selected_idx = 0;
    }

    let down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
    let up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
    let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    if !hits.is_empty() {
        if down {
            state.palette_selected_idx = (state.palette_selected_idx + 1) % hits.len();
        }
        if up {
            state.palette_selected_idx =
                (state.palette_selected_idx + hits.len() - 1) % hits.len();
        }
    }

    let mut activate: Option<PaletteTarget> = None;
    if enter && !hits.is_empty() {
        activate = Some(hits[state.palette_selected_idx].target.clone());
    }

    let modal_w = (screen.width() - 48.0).clamp(340.0, 680.0);
    let modal_h = (screen.height() - 110.0).clamp(280.0, 480.0);
    let modal_pos = Pos2::new(
        screen.center().x - modal_w * 0.5,
        screen.top() + (screen.height() - modal_h) * 0.25,
    );

    egui::Area::new(egui::Id::new("mc_cmd_palette_modal"))
        .order(egui::Order::Foreground)
        .fixed_pos(modal_pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgba_unmultiplied(
                    palette.bg_panel.r(),
                    palette.bg_panel.g(),
                    palette.bg_panel.b(),
                    248,
                ))
                .stroke(Stroke::new(1.5_f32, palette.accent_primary))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(modal_w);
                    ui.set_max_height(modal_h);

                    ui.horizontal(|ui| {
                        ui.add(atlas.ui_image("spyglass", 20.0));
                        ui.label(
                            RichText::new(t(lang, UiKey::CommandPaletteTitle))
                                .strong()
                                .size(14.5)
                                .color(palette.text_primary),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                state.search_palette_open = false;
                            }
                            ui.label(
                                RichText::new(&active_ver)
                                    .size(11.0)
                                    .color(palette.accent_xp),
                            );
                        });
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let te = ui.add_sized(
                            [ui.available_width() - 68.0, 30.0],
                            egui::TextEdit::singleline(&mut state.palette_query)
                                .hint_text(t(lang, UiKey::SearchPlaceholder)),
                        );
                        if state.focus_search_next_frame {
                            te.request_focus();
                            state.focus_search_next_frame = false;
                        }
                        if te.changed() {
                            state.palette_selected_idx = 0;
                        }
                        if ui
                            .add_enabled(
                                !state.palette_query.is_empty(),
                                egui::Button::new(t(lang, UiKey::ClearSearch)),
                            )
                            .clicked()
                        {
                            state.palette_query.clear();
                            state.palette_selected_idx = 0;
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(t(lang, UiKey::CommandPaletteHint))
                            .size(10.5)
                            .color(palette.text_muted),
                    );
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height((modal_h - 115.0).max(150.0))
                        .show(ui, |ui| {
                            if hits.is_empty() {
                                ui.add_space(16.0);
                                ui.label(
                                    RichText::new(t(lang, UiKey::CommandPaletteEmpty))
                                        .color(palette.text_muted),
                                );
                                return;
                            }

                            for (idx, hit) in hits.iter().enumerate() {
                                let selected = idx == state.palette_selected_idx;
                                let row_resp = egui::Frame::new()
                                    .fill(if selected {
                                        palette.bg_card_hover
                                    } else {
                                        palette.bg_card
                                    })
                                    .stroke(Stroke::new(
                                        if selected { 1.5_f32 } else { 1.0_f32 },
                                        if selected {
                                            hit.accent
                                        } else {
                                            palette.border_subtle
                                        },
                                    ))
                                    .corner_radius(CornerRadius::same(5))
                                    .inner_margin(Margin::symmetric(8, 5))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.add(atlas.ui_image(&hit.icon, 22.0));
                                            ui.label(
                                                RichText::new(&hit.title)
                                                    .strong()
                                                    .size(12.5)
                                                    .color(palette.text_primary),
                                            );
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(hit.module_label)
                                                            .size(10.5)
                                                            .strong()
                                                            .color(hit.accent),
                                                    );
                                                },
                                            );
                                        });
                                    })
                                    .response
                                    .interact(Sense::click());

                                if row_resp.hovered() {
                                    state.palette_selected_idx = idx;
                                }
                                if row_resp.clicked() {
                                    activate = Some(hit.target.clone());
                                }
                                ui.add_space(3.0);
                            }
                        });
                });
        });

    if let Some(target) = activate {
        state.search_palette_open = false;
        state.search_query.clear();
        match target {
            PaletteTarget::Gear(id) => {
                state.selected_gear_id = id;
                state.switch_tab(AppTab::Gear);
            }
            PaletteTarget::Recipe(id) => {
                state.open_recipe_item(&id, RecipeViewMode::RecipesFor);
            }
            PaletteTarget::Farm(id) => {
                state.selected_farm_id = id;
                state.switch_tab(AppTab::Farms);
            }
            PaletteTarget::Dupe(id) => {
                state.selected_dupe_id = id;
                state.switch_tab(AppTab::Dupes);
            }
            PaletteTarget::Tip(id) => {
                state.selected_tip_id = id;
                state.switch_tab(AppTab::ProTips);
            }
        }
        audio.play(SoundCue::Click);
    }
}

/// Shared clean inline search box rendered inside each catalog tab so users can still filter lists locally.
pub fn render_inline_tab_search(
    ui: &mut egui::Ui,
    state: &mut AppState,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
) {
    let lang = state.prefs.language;
    ui.horizontal(|ui| {
        ui.add(atlas.ui_image("spyglass", 16.0));
        let w = (ui.available_width() - if state.search_query.is_empty() { 4.0 } else { 32.0 }).max(80.0);
        ui.add_sized(
            [w, 22.0],
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text(t(lang, UiKey::LocalFilterHint)),
        );
        if !state.search_query.is_empty() {
            if ui
                .button(RichText::new("×").color(palette.text_muted))
                .on_hover_text(t(lang, UiKey::ClearSearch))
                .clicked()
            {
                state.search_query.clear();
                audio.play(SoundCue::Click);
            }
        }
    });
    ui.add_space(4.0);
}

/// Renders collapsible Pinned Favorites Rail (shown when `prefs.favorites_open` and window width allows).
pub fn render_favorites_rail(
    ctx: &egui::Context,
    state: &mut AppState,
    versions: &VersionRegistry,
    db: &Database,
    recipes: &RecipeDatabase,
    palette: McPalette,
    atlas: &ItemAtlas,
    audio: &AudioEngine,
) {
    if !state.prefs.favorites_open || ctx.screen_rect().width() < 860.0 {
        return;
    }
    let lang = state.prefs.language;
    let active_ver = versions
        .resolve_valid_id(&state.prefs.selected_version)
        .to_string();

    egui::SidePanel::right("mc_favorites_rail")
        .resizable(false)
        .default_width(216.0)
        .frame(
            egui::Frame::new()
                .fill(palette.bg_panel)
                .inner_margin(Margin::symmetric(10, 10))
                .stroke(Stroke::new(1.0_f32, palette.border_subtle)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(atlas.ui_image("nether_star", 16.0));
                ui.label(
                    RichText::new(t(lang, UiKey::FavoritesTitle))
                        .strong()
                        .size(13.0)
                        .color(palette.accent_gold),
                );
            });
            ui.separator();

            if state.prefs.favorite_items.is_empty()
                && state.prefs.favorite_gear.is_empty()
                && state.prefs.favorite_farms.is_empty()
            {
                ui.label(
                    RichText::new(t(lang, UiKey::FavoritesEmpty))
                        .size(11.5)
                        .color(palette.text_muted),
                );
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Pinned JEI Items
                let items_clone = state.prefs.favorite_items.clone();
                if !items_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabRecipes))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_item: Option<String> = None;
                    for id in &items_clone {
                        if !recipes.item_active_in(id, versions, &active_ver) {
                            continue;
                        }
                        let name = recipes.item_name(id, lang);
                        ui.horizontal(|ui| {
                            let (slot_r, sresp) =
                                ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                            paint_mc_slot(
                                ui.painter(),
                                slot_r,
                                palette,
                                atlas,
                                Some(id),
                                1,
                                state.active_tab == AppTab::Recipes
                                    && state.selected_recipe_item == *id,
                                sresp.hovered(),
                                None,
                            );
                            if sresp.clicked() {
                                state.open_recipe_item(id, RecipeViewMode::RecipesFor);
                                audio.play(SoundCue::Click);
                            }
                            let lbl_resp = ui.add(
                                egui::Button::new(
                                    RichText::new(&name).size(11.5).color(palette.text_primary),
                                )
                                .frame(false)
                                .truncate(),
                            );
                            if lbl_resp.clicked() {
                                state.open_recipe_item(id, RecipeViewMode::RecipesFor);
                                audio.play(SoundCue::Click);
                            }
                            if ui.small_button("x").clicked() {
                                remove_item = Some(id.clone());
                            }
                        });
                    }
                    if let Some(rm) = remove_item {
                        state.prefs.toggle_favorite_item(&rm);
                        audio.play(SoundCue::Click);
                    }
                    ui.add_space(8.0);
                }

                // Pinned God Gear
                let gear_clone = state.prefs.favorite_gear.clone();
                if !gear_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabGear))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_gear: Option<String> = None;
                    for gid in &gear_clone {
                        if let Some(g) = db.gear.iter().find(|x| {
                            x.id == *gid
                                && versions.is_active(
                                    x.since_version.as_deref(),
                                    x.until_version.as_deref(),
                                    &active_ver,
                                )
                        }) {
                            let res = g.resolved(&active_ver);
                            ui.horizontal(|ui| {
                                ui.add(atlas.ui_image(&g.icon, 20.0));
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(res.name.get(lang))
                                                .size(11.5)
                                                .color(palette.accent_primary),
                                        )
                                        .frame(false)
                                        .truncate(),
                                    )
                                    .clicked()
                                {
                                    state.selected_gear_id = g.id.clone();
                                    state.switch_tab(AppTab::Gear);
                                    audio.play(SoundCue::Click);
                                }
                                if ui.small_button("x").clicked() {
                                    remove_gear = Some(gid.clone());
                                }
                            });
                        }
                    }
                    if let Some(rm) = remove_gear {
                        state.prefs.toggle_favorite_gear(&rm);
                        audio.play(SoundCue::Click);
                    }
                    ui.add_space(8.0);
                }

                // Pinned Farms
                let farms_clone = state.prefs.favorite_farms.clone();
                if !farms_clone.is_empty() {
                    ui.label(
                        RichText::new(t(lang, UiKey::TabFarms))
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);
                    let mut remove_farm: Option<String> = None;
                    for fid in &farms_clone {
                        if let Some(f) = db.farms.iter().find(|x| {
                            x.id == *fid
                                && versions.is_active(
                                    x.since_version.as_deref(),
                                    x.until_version.as_deref(),
                                    &active_ver,
                                )
                        }) {
                            let res = f.resolved(&active_ver);
                            ui.horizontal(|ui| {
                                let icon = res.resolved_icon();
                                ui.add(atlas.ui_image(icon, 18.0));
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(res.title.get(lang))
                                                .size(11.5)
                                                .color(palette.accent_xp),
                                        )
                                        .frame(false)
                                        .truncate(),
                                    )
                                    .clicked()
                                {
                                    state.selected_farm_id = f.id.clone();
                                    state.switch_tab(AppTab::Farms);
                                    audio.play(SoundCue::Click);
                                }
                                if ui.small_button("x").clicked() {
                                    remove_farm = Some(fid.clone());
                                }
                            });
                        }
                    }
                    if let Some(rm) = remove_farm {
                        state.prefs.toggle_favorite_farm(&rm);
                        audio.play(SoundCue::Click);
                    }
                }
            });
        });
}
