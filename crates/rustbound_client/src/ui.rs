//! In-game egui overlay: docking IDE, telemetry, mission objectives, and compiler controls.

use crate::dock::{RustboundTabViewer, TabContext};
use crate::menu::apply_ferric_theme;
use crate::{AppState, GameSession};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_dock::DockArea;

/// Main in-game UI system rendered into the primary egui context pass.
pub fn in_game_ui_system(
    mut contexts: EguiContexts,
    mut game_session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    apply_ferric_theme(ctx);
    let f5_pressed = keyboard.just_pressed(KeyCode::F5);

    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "in_game_viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    // ── Top Bar Toolbar ───────────────────────────────────────────────────────
    let mut return_to_menu = false;
    let mut trigger_compile = false;

    egui::Panel::top("in_game_toolbar")
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(14, 16, 20))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 52, 60))),
        )
        .show(&mut viewport_ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("RUSTBOUND")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(230, 97, 44))
                        .strong(),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(&game_session.session.mission.title)
                        .size(13.0)
                        .color(egui::Color32::from_rgb(175, 185, 195)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);

                    let tick = game_session.session.sim_runner.current_tick();
                    ui.label(
                        egui::RichText::new(format!("TICK {tick:06}"))
                            .size(12.0)
                            .color(egui::Color32::from_rgb(80, 180, 110))
                            .monospace(),
                    );

                    ui.separator();

                    let deploy_btn = egui::Button::new(
                        egui::RichText::new("⚡ Deploy [F5]")
                            .size(13.0)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(180, 70, 30))
                    .min_size(egui::vec2(120.0, 26.0));

                    if ui.add(deploy_btn).clicked() || f5_pressed {
                        trigger_compile = true;
                    }

                    ui.separator();

                    if ui
                        .button(
                            egui::RichText::new("◀ Menu")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(150, 160, 175)),
                        )
                        .clicked()
                    {
                        return_to_menu = true;
                    }
                });
            });
        });

    if return_to_menu {
        next_state.set(AppState::MainMenu);
    }
    if trigger_compile {
        trigger_compilation_and_deploy(&mut game_session);
    }

    // ── Docking IDE Panel (Right Side) ────────────────────────────────────────
    let telemetry_map = game_session.session.collect_telemetry_snapshot();
    let terminal_logs = game_session.session.terminal_logs.clone();

    egui::Panel::right("ide_dock_panel")
        .default_size(580.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(16, 18, 22)))
        .show(&mut viewport_ui, |ui| {
            let session = &mut game_session.session;
            let ctx_data = TabContext {
                editor: &mut session.editor,
                inspector: &mut session.inspector,
                terminal_logs: &terminal_logs,
                telemetry_map: &telemetry_map,
            };
            let mut tab_viewer = RustboundTabViewer { context: ctx_data };

            DockArea::new(&mut session.dock_state)
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut tab_viewer);
        });

    // ── Objectives Floating Window (Top-Left) ─────────────────────────────────
    egui::Window::new("DIRECTIVE OBJECTIVES")
        .collapsible(true)
        .resizable(false)
        .anchor(egui::Align2::LEFT_TOP, [10.0, 48.0])
        .min_width(260.0)
        .show(ctx, |ui| {
            for obj in &game_session.session.mission.objectives {
                let (icon, color) = if obj.completed {
                    ("✓", egui::Color32::from_rgb(80, 200, 100))
                } else {
                    ("○", egui::Color32::from_rgb(190, 195, 200))
                };
                ui.label(
                    egui::RichText::new(format!("{icon}  {}", obj.description))
                        .size(12.0)
                        .color(color),
                );
            }
        });

    // ── Mission Briefing Modal ────────────────────────────────────────────────
    if game_session.show_mission_briefing {
        let mut open = true;
        egui::Window::new("⚡ FERRIC ARBITER — MISSION DIRECTIVE")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .min_width(540.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&game_session.session.mission.title)
                            .size(18.0)
                            .color(egui::Color32::from_rgb(230, 97, 44))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Location: {}",
                            game_session.session.mission.environment
                        ))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(105, 185, 210)),
                    );

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&game_session.session.mission.briefing)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(205, 210, 215)),
                            );
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new("OBJECTIVES:")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(140, 150, 160))
                            .strong(),
                    );
                    for obj in &game_session.session.mission.objectives {
                        ui.label(
                            egui::RichText::new(format!("• {}", obj.description))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(190, 200, 210)),
                        );
                    }

                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        let btn = egui::Button::new(
                            egui::RichText::new("COMMENCE OPERATION  ▶")
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(180, 70, 30))
                        .min_size(egui::vec2(260.0, 36.0));

                        if ui.add(btn).clicked() {
                            game_session.show_mission_briefing = false;
                        }
                    });
                    ui.add_space(6.0);
                });
            });
        if !open {
            game_session.show_mission_briefing = false;
        }
    }

    // ── Mission Complete Modal ────────────────────────────────────────────────
    if game_session.show_mission_complete {
        egui::Window::new("✅ SECTOR RESTORED")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .min_width(420.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("MISSION ACCOMPLISHED")
                            .size(22.0)
                            .color(egui::Color32::from_rgb(80, 200, 100))
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(&game_session.session.mission.title)
                            .size(15.0)
                            .color(egui::Color32::from_rgb(230, 97, 44)),
                    );

                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new(
                            "The Ferric Arbiter has validated firmware execution and safety guarantees.",
                        )
                        .size(12.0)
                        .color(egui::Color32::from_rgb(180, 190, 200)),
                    );

                    ui.add_space(20.0);

                    let next_id = game_session.current_mission + 1;
                    if next_id <= 3 {
                        let next_btn = egui::Button::new(
                            egui::RichText::new(format!("Proceed to Mission {next_id:02}  ▶"))
                                .size(14.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(45, 120, 60))
                        .min_size(egui::vec2(280.0, 38.0));

                        if ui.add(next_btn).clicked() {
                            game_session.setup_mission(next_id);
                        }
                        ui.add_space(8.0);
                    }

                    let menu_btn = egui::Button::new(
                        egui::RichText::new("← Return to Reclamation Directives")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(170, 180, 190)),
                    )
                    .fill(egui::Color32::from_rgb(32, 36, 44))
                    .min_size(egui::vec2(280.0, 30.0));

                    if ui.add(menu_btn).clicked() {
                        game_session.show_mission_complete = false;
                        next_state.set(AppState::MainMenu);
                    }
                    ui.add_space(8.0);
                });
            });
    }
}

/// Triggers real rustc compilation of the current editor code and deploys firmware to Machine #0.
pub fn trigger_compilation_and_deploy(game: &mut GameSession) {
    game.session.terminal_logs.push((
        0,
        String::from(
            "Rustbound Compiler: Compiling player firmware (target: wasm32-unknown-unknown)...",
        ),
    ));

    match game.session.compile_current_firmware() {
        Ok(wasm_bytes) => {
            let target_machine_id = 0;
            match game
                .session
                .deploy_firmware_bytes(target_machine_id, &wasm_bytes)
            {
                Ok(()) => {
                    game.session.terminal_logs.push((
                        0,
                        String::from(
                            "✓ Firmware deployed successfully to Machine #0. Running in Wasmi VM at 20 Hz.",
                        ),
                    ));
                }
                Err(e) => {
                    game.session
                        .terminal_logs
                        .push((2, format!("✗ Deploy failed: {e}")));
                }
            }
        }
        Err(e) => {
            game.session
                .terminal_logs
                .push((2, format!("✗ Build failed: {e}")));
        }
    }
}
