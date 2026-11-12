//! Main menu interface rendered with Ferric Brutalism visual style.

use crate::{AppState, GameSession};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

/// Marker component for the 2D menu camera.
#[derive(Component)]
pub struct MenuCamera;

/// Spawns the 2D camera used in the main menu.
pub fn setup_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera));
}

/// Cleans up the 2D camera when transitioning away from the main menu.
pub fn cleanup_menu_camera(mut commands: Commands) {
    commands.despawn_all::<With<MenuCamera>>();
}

/// Main menu UI pass rendered via egui.
pub fn menu_ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_session: ResMut<GameSession>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    apply_ferric_theme(ctx);

    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "menu_viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(18, 20, 24)))
        .show(&mut viewport_ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                // Branding Header
                ui.label(
                    egui::RichText::new("RUSTBOUND")
                        .size(64.0)
                        .color(egui::Color32::from_rgb(230, 97, 44))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("THE SYSTEMS FRONTIER")
                        .size(20.0)
                        .color(egui::Color32::from_rgb(180, 190, 205))
                        .extra_letter_spacing(5.0),
                );

                ui.add_space(18.0);
                ui.label(
                    egui::RichText::new("Think in Rust. Build beyond limits.")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(105, 185, 210))
                        .italics(),
                );

                ui.add_space(45.0);
                ui.separator();
                ui.add_space(30.0);

                // Mission Selection List
                ui.label(
                    egui::RichText::new("RECLAMATION DIRECTIVES")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(140, 150, 160))
                        .strong(),
                );
                ui.add_space(14.0);

                for mission_id in 1u32..=6 {
                    let label = match mission_id {
                        1 => "▶  Mission 01 — Cold Boot (Aux Generator Ignition)",
                        2 => "▶  Mission 02 — Subterranean Extraction (Thermal Core)",
                        3 => "▶  Mission 03 — Single Owner Custody (Affine Logistics)",
                        4 => "▶  Mission 04 — Immutable Interlock (Depth Registers)",
                        5 => "▶  Mission 05 — Thermal Governor (Expression Returns)",
                        6 => "▶  Mission 06 — Automated Gantry (Multi-Machine Routing)",
                        _ => "▶  Mission",
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(label)
                            .size(15.0)
                            .color(egui::Color32::WHITE),
                    )
                    .min_size(egui::vec2(440.0, 44.0))
                    .fill(egui::Color32::from_rgb(32, 36, 44))
                    .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(75, 85, 95)));

                    if ui.add(btn).clicked() {
                        game_session.setup_mission(mission_id);
                        next_state.set(AppState::InGame);
                    }
                    ui.add_space(8.0);
                }

                ui.add_space(25.0);
                ui.separator();
                ui.add_space(16.0);

                // Exit button
                let quit_btn = egui::Button::new(
                    egui::RichText::new("Abort Reconnection (Quit)")
                        .size(13.0)
                        .color(egui::Color32::from_rgb(200, 80, 70)),
                )
                .min_size(egui::vec2(200.0, 32.0))
                .fill(egui::Color32::from_rgb(25, 20, 20));

                if ui.add(quit_btn).clicked() {
                    exit_writer.write(AppExit::Success);
                }

                ui.add_space(45.0);

                // Footer Protocol Info
                ui.label(
                    egui::RichText::new(
                        "PLANETARY SECURITY: FERRIC ARBITER ONLINE // 20 HZ DETERMINISTIC",
                    )
                    .size(10.0)
                    .color(egui::Color32::from_rgb(70, 130, 110))
                    .monospace(),
                );
            });
        });
}

/// Applies the Ferric Brutalism dark theme with graphite, industrial slate, and rust accents.
pub fn apply_ferric_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(18, 20, 24);
    visuals.window_fill = egui::Color32::from_rgb(22, 25, 30);
    visuals.faint_bg_color = egui::Color32::from_rgb(26, 29, 35);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 14, 16);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 27, 32);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 36, 44);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 52, 64);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(230, 97, 44);
    visuals.selection.bg_fill = egui::Color32::from_rgb(180, 70, 30);
    ctx.set_visuals(visuals);
}
