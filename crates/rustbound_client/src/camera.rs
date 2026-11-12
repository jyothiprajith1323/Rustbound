//! Isometric CAD-style orbit camera for the industrial Aethelgard environment.

use crate::environment::EnvironmentEntity;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

/// Marker and parameters for the 3D industrial viewport camera.
#[derive(Component)]
pub struct GameCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            yaw: core::f32::consts::FRAC_PI_4,
            pitch: 0.65,
            distance: 18.0,
            target: Vec3::new(0.0, 0.5, 0.0),
        }
    }
}

/// Spawns the main 3D camera and industrial lighting rigs.
pub fn spawn_camera(mut commands: Commands) {
    let cam = GameCamera::default();
    let transform = compute_camera_transform(&cam);

    // Camera entity with ambient light override
    commands.spawn((
        Camera3d::default(),
        transform,
        cam,
        AmbientLight {
            color: Color::srgb(0.25, 0.40, 0.48),
            brightness: 180.0,
            affects_lightmapped_meshes: true,
        },
        EnvironmentEntity,
    ));

    // Directional light — warm amber industrial fill
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.82, 0.65),
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 22.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        EnvironmentEntity,
    ));
}

/// Computes the camera Transform from orbit parameters.
pub fn compute_camera_transform(cam: &GameCamera) -> Transform {
    let x = cam.target.x + cam.distance * cam.yaw.cos() * cam.pitch.cos();
    let y = cam.target.y + cam.distance * cam.pitch.sin();
    let z = cam.target.z + cam.distance * cam.yaw.sin() * cam.pitch.cos();
    Transform::from_xyz(x, y, z).looking_at(cam.target, Vec3::Y)
}

/// Orbit, pan, and zoom camera controller system.
pub fn orbit_camera_system(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut GameCamera, &mut Transform)>,
) {
    let Ok((mut cam, mut transform)) = query.single_mut() else {
        return;
    };

    // Zoom via mouse scroll wheel
    if mouse_scroll.delta.y.abs() > 0.01 {
        cam.distance = (cam.distance - mouse_scroll.delta.y * 1.5).clamp(4.0, 50.0);
    }

    // Orbit via Right Mouse Button drag
    if mouse_buttons.pressed(MouseButton::Right) {
        cam.yaw -= mouse_motion.delta.x * 0.005;
        cam.pitch = (cam.pitch - mouse_motion.delta.y * 0.005).clamp(0.12, 1.45);
    }

    // Pan via Middle Mouse Button drag
    if mouse_buttons.pressed(MouseButton::Middle) {
        let right = transform.right().as_vec3();
        let up = Vec3::Y;
        cam.target += right * (-mouse_motion.delta.x * 0.015);
        cam.target += up * (mouse_motion.delta.y * 0.015);
    }

    // Pan via keyboard (WASD / Arrows)
    let pan_speed = 0.12;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        cam.target.z -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        cam.target.z += pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        cam.target.x -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        cam.target.x += pan_speed;
    }

    *transform = compute_camera_transform(&cam);
}
