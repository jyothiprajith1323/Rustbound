//! Rustbound: The Systems Frontier — application binary entry point.

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use rustbound_client::{
    AppState, GameSession, cleanup_environment, cleanup_menu_camera, in_game_ui_system,
    machine_pick_system, menu_ui_system, orbit_camera_system, setup_menu_camera, spawn_camera,
    spawn_environment, spawn_machines,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Rustbound: The Systems Frontier".into(),
            resolution: (1600u32, 900u32).into(),
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(EguiPlugin::default());

    app.init_state::<AppState>();
    app.init_resource::<GameSession>();

    // Main Menu State Lifecycle
    app.add_systems(OnEnter(AppState::MainMenu), setup_menu_camera);
    app.add_systems(OnExit(AppState::MainMenu), cleanup_menu_camera);

    // In-Game State Lifecycle
    app.add_systems(
        OnEnter(AppState::InGame),
        (spawn_camera, spawn_environment, setup_in_game_machines),
    );
    app.add_systems(OnExit(AppState::InGame), cleanup_environment);

    // Dynamic In-Game Systems (20 Hz simulation + camera orbit + raycast picking)
    app.add_systems(
        Update,
        (
            orbit_camera_system,
            machine_pick_system,
            simulation_20hz_system,
        )
            .run_if(in_state(AppState::InGame)),
    );

    // Egui Primary UI Passes
    app.add_systems(
        EguiPrimaryContextPass,
        (
            menu_ui_system.run_if(in_state(AppState::MainMenu)),
            in_game_ui_system.run_if(in_state(AppState::InGame)),
        ),
    );

    app.run();
}

/// Spawns the 3D machine meshes for the active mission.
fn setup_in_game_machines(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    game: Res<GameSession>,
) {
    spawn_machines(commands, meshes, materials, game.current_mission);
}

/// Ticks the headless ECS simulation at 20 Hz (50 ms per tick).
fn simulation_20hz_system(time: Res<Time>, mut game: ResMut<GameSession>) {
    game.sim_timer += time.delta_secs();
    while game.sim_timer >= 0.05 {
        game.sim_timer -= 0.05;
        game.session.step_simulation();

        if game.session.is_mission_completed() && !game.mission_complete {
            game.mission_complete = true;
            game.show_mission_complete = true;
        }
    }
}
