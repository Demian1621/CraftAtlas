use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Margin, Pos2, RichText, Sense, Stroke,
    ViewportCommand,
};

use crate::audio::{AudioEngine, SoundCue};
use crate::i18n::{t, UiKey};
use crate::state::AppState;
use crate::ui::theme::McPalette;
use crate::updater::{
    is_safe_https_url, relaunch_updated_binary, InstallState, UpdateStatus, CURRENT_VERSION,
};

/// Centered modal for SHA-256 verified in-app updates (fail-closed, atomic swap where supported).
pub fn render_updater_modal(
    ctx: &egui::Context,
    state: &mut AppState,
    palette: McPalette,
    audio: &AudioEngine,
) {
    if !state.update_modal_open {
        return;
    }

    let info = match state.updater.status() {
        UpdateStatus::Available(i) => i.clone(),
        _ => {
            state.update_modal_open = false;
            return;
        }
    };
    let lang = state.prefs.language;

    if ctx.input(|i| i.key_pressed(egui::Key::Escape))
        && !matches!(state.updater.install_state(), InstallState::Downloading { .. })
    {
        state.update_modal_open = false;
        return;
    }

    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("mc_updater_backdrop"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (_r, bg_resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter()
                .rect_filled(screen, 0.0, Color32::from_black_alpha(165));
            if bg_resp.clicked()
                && !matches!(state.updater.install_state(), InstallState::Downloading { .. })
            {
                state.update_modal_open = false;
            }
        });

    let modal_w = 480.0_f32.min(screen.width() - 32.0);
    let pos = Pos2::new(
        screen.center().x - modal_w * 0.5,
        (screen.center().y - 165.0).max(48.0),
    );

    egui::Area::new(egui::Id::new("mc_updater_modal"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(palette.bg_panel)
                .stroke(Stroke::new(1.5_f32, palette.accent_xp))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(modal_w);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(t(lang, UiKey::UpdaterModalTitle))
                                .size(15.0)
                                .strong()
                                .color(palette.accent_xp),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("v{} -> v{}", CURRENT_VERSION, info.version))
                                    .size(12.0)
                                    .monospace()
                                    .strong()
                                    .color(palette.accent_gold),
                            );
                        });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(t(lang, UiKey::UpdaterSha256Badge))
                            .size(11.0)
                            .color(palette.accent_primary),
                    );
                    ui.separator();

                    if !info.notes_summary.is_empty() {
                        egui::Frame::new()
                            .fill(palette.bg_card)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(&info.notes_summary)
                                        .size(11.5)
                                        .color(palette.text_primary),
                                );
                            });
                        ui.add_space(10.0);
                    }

                    let install_snapshot = state.updater.install_state().clone();
                    match install_snapshot {
                        InstallState::Idle => {
                            ui.horizontal(|ui| {
                                let primary = egui::Button::new(
                                    RichText::new(t(lang, UiKey::UpdaterDownloadInstallBtn))
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(palette.accent_xp.gamma_multiply(0.9))
                                .corner_radius(CornerRadius::same(6));
                                if ui.add(primary).clicked() {
                                    audio.play(SoundCue::Click);
                                    state.updater.start_verified_install(info.clone());
                                }
                                if ui.button(t(lang, UiKey::UpdaterOpenGitHubBtn)).clicked() {
                                    audio.play(SoundCue::Click);
                                    if is_safe_https_url(&info.html_url) {
                                        ctx.open_url(egui::OpenUrl::new_tab(info.html_url.clone()));
                                    }
                                }
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button(t(lang, UiKey::UpdaterCloseBtn)).clicked() {
                                        audio.play(SoundCue::Click);
                                        state.update_modal_open = false;
                                    }
                                });
                            });
                        }
                        InstallState::Downloading { asset_name } => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(
                                    RichText::new(t(lang, UiKey::UpdaterDownloadingLabel))
                                        .size(12.0)
                                        .color(palette.text_primary),
                                );
                            });
                            ui.label(
                                RichText::new(asset_name)
                                    .size(11.0)
                                    .monospace()
                                    .color(palette.text_muted),
                            );
                        }
                        InstallState::ReadyToRestart {
                            relaunch_path,
                            saved_only_path,
                            verified_sha256,
                            ..
                        } => {
                            let short_hash = &verified_sha256[..16.min(verified_sha256.len())];
                            ui.label(
                                RichText::new(format!("SHA-256 OK: {}...", short_hash))
                                    .size(11.0)
                                    .monospace()
                                    .strong()
                                    .color(palette.accent_xp),
                            );
                            ui.add_space(4.0);
                            if let Some(exe_path) = relaunch_path {
                                ui.label(
                                    RichText::new(t(lang, UiKey::UpdaterVerifiedReadyLabel))
                                        .size(12.0)
                                        .color(palette.text_primary),
                                );
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let btn = egui::Button::new(
                                        RichText::new(t(lang, UiKey::UpdaterRestartNowBtn))
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(palette.accent_xp)
                                    .corner_radius(CornerRadius::same(6));
                                    if ui.add(btn).clicked() {
                                        audio.play(SoundCue::LevelUp);
                                        if relaunch_updated_binary(&exe_path) {
                                            ctx.send_viewport_cmd(ViewportCommand::Close);
                                        }
                                    }
                                    if ui.button(t(lang, UiKey::UpdaterCloseBtn)).clicked() {
                                        state.update_modal_open = false;
                                    }
                                });
                            } else if let Some(saved) = saved_only_path {
                                ui.label(
                                    RichText::new(t(lang, UiKey::UpdaterSavedToDownloadsLabel))
                                        .size(12.0)
                                        .color(palette.text_primary),
                                );
                                ui.label(
                                    RichText::new(saved.display().to_string())
                                        .size(11.0)
                                        .monospace()
                                        .color(palette.accent_gold),
                                );
                                ui.add_space(6.0);
                                if ui.button(t(lang, UiKey::UpdaterCloseBtn)).clicked() {
                                    state.update_modal_open = false;
                                }
                            }
                        }
                        InstallState::Failed(err_msg) => {
                            ui.label(
                                RichText::new(err_msg)
                                    .size(11.5)
                                    .color(palette.accent_danger),
                            );
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button(t(lang, UiKey::UpdaterOpenGitHubBtn)).clicked() {
                                    audio.play(SoundCue::Click);
                                    if is_safe_https_url(&info.html_url) {
                                        ctx.open_url(egui::OpenUrl::new_tab(info.html_url.clone()));
                                    }
                                }
                                if ui.button(t(lang, UiKey::UpdaterCloseBtn)).clicked() {
                                    state.updater.reset_install_state();
                                    state.update_modal_open = false;
                                }
                            });
                        }
                    }
                });
        });
}
