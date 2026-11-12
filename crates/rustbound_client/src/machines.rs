//! 3D Machine mesh generation and interactive raycast picking.

use crate::camera::GameCamera;
use crate::environment::EnvironmentEntity;
use bevy::prelude::*;

/// Identifies an in-world machine visual entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct MachineMarker {
    pub machine_id: u32,
    pub machine_type: MachineVisualType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineVisualType {
    Generator,
    QuarryDrill,
    ConveyorBelt,
    StorageBay,
}

/// Spawns 3D machines according to the currently active mission.
pub fn spawn_machines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mission_id: u32,
) {
    match mission_id {
        1 => {
            // Mission 1: Auxiliary Generator (ID: 0)
            spawn_generator(
                &mut commands,
                &mut meshes,
                &mut materials,
                0,
                Vec3::new(0.0, 0.35, 0.0),
            );
        }
        2 => {
            // Mission 2: Quarry Excavator Drill (ID: 0)
            spawn_drill(
                &mut commands,
                &mut meshes,
                &mut materials,
                0,
                Vec3::new(0.0, 0.35, 0.0),
            );
        }
        3 => {
            // Mission 3: Mining Chain — Drill (0), Conveyor (1), Storage (2)
            spawn_drill(
                &mut commands,
                &mut meshes,
                &mut materials,
                0,
                Vec3::new(-5.0, 0.35, 0.0),
            );
            spawn_conveyor(
                &mut commands,
                &mut meshes,
                &mut materials,
                1,
                Vec3::new(0.0, 0.35, 0.0),
            );
            spawn_storage(
                &mut commands,
                &mut meshes,
                &mut materials,
                2,
                Vec3::new(5.0, 0.35, 0.0),
            );
        }
        _ => {
            spawn_generator(
                &mut commands,
                &mut meshes,
                &mut materials,
                0,
                Vec3::new(0.0, 0.35, 0.0),
            );
        }
    }
}

fn spawn_generator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: u32,
    pos: Vec3,
) {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.20, 0.24),
        metallic: 0.85,
        perceptual_roughness: 0.35,
        ..default()
    });
    let accent_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.38, 0.17),
        emissive: LinearRgba::new(0.92, 0.38, 0.17, 3.0),
        ..default()
    });

    // Primary cylindrical turbine casing
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, 2.6))),
        MeshMaterial3d(body_mat.clone()),
        Transform::from_translation(pos + Vec3::Y * 1.3),
        MachineMarker {
            machine_id: id,
            machine_type: MachineVisualType::Generator,
        },
        EnvironmentEntity,
    ));

    // Power status glow collar
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.06, 0.18))),
        MeshMaterial3d(accent_mat),
        Transform::from_translation(pos + Vec3::Y * 2.5),
        EnvironmentEntity,
    ));

    // High-temperature exhaust manifold
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.24, 1.4))),
        MeshMaterial3d(body_mat),
        Transform::from_translation(pos + Vec3::new(0.6, 3.1, 0.0)),
        EnvironmentEntity,
    ));
}

fn spawn_drill(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: u32,
    pos: Vec3,
) {
    let chassis_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.19, 0.14),
        metallic: 0.75,
        perceptual_roughness: 0.5,
        ..default()
    });
    let bit_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.45, 0.12),
        metallic: 0.95,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Main drill rig housing
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 2.2, 1.8))),
        MeshMaterial3d(chassis_mat.clone()),
        Transform::from_translation(pos + Vec3::Y * 1.1),
        MachineMarker {
            machine_id: id,
            machine_type: MachineVisualType::QuarryDrill,
        },
        EnvironmentEntity,
    ));

    // Heavy rotary drill bit (inverted cone)
    commands.spawn((
        Mesh3d(meshes.add(Cone::new(0.45, 1.6))),
        MeshMaterial3d(bit_mat),
        Transform::from_translation(pos + Vec3::new(0.0, -0.4, 0.0))
            .with_rotation(Quat::from_rotation_x(core::f32::consts::PI)),
        EnvironmentEntity,
    ));

    // Radiator cooling fins
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 1.0, 1.6))),
        MeshMaterial3d(chassis_mat),
        Transform::from_translation(pos + Vec3::new(1.05, 1.4, 0.0)),
        EnvironmentEntity,
    ));
}

fn spawn_conveyor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: u32,
    pos: Vec3,
) {
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.25, 0.28),
        metallic: 0.65,
        perceptual_roughness: 0.5,
        ..default()
    });
    let belt_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.09),
        metallic: 0.2,
        perceptual_roughness: 0.9,
        ..default()
    });

    // Conveyor chassis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.2, 0.35, 1.4))),
        MeshMaterial3d(frame_mat.clone()),
        Transform::from_translation(pos + Vec3::Y * 0.18),
        MachineMarker {
            machine_id: id,
            machine_type: MachineVisualType::ConveyorBelt,
        },
        EnvironmentEntity,
    ));

    // Rubber conveyor transit track
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.8, 0.12, 1.0))),
        MeshMaterial3d(belt_mat),
        Transform::from_translation(pos + Vec3::Y * 0.4),
        EnvironmentEntity,
    ));

    // End drive rollers
    for x_offset in [-2.4f32, 2.4] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.22, 1.2))),
            MeshMaterial3d(frame_mat.clone()),
            Transform::from_translation(pos + Vec3::new(x_offset, 0.35, 0.0))
                .with_rotation(Quat::from_rotation_z(core::f32::consts::FRAC_PI_2)),
            EnvironmentEntity,
        ));
    }
}

fn spawn_storage(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    id: u32,
    pos: Vec3,
) {
    let silo_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.16, 0.19),
        metallic: 0.7,
        perceptual_roughness: 0.55,
        ..default()
    });
    let hazard_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.72, 0.12),
        emissive: LinearRgba::new(0.4, 0.32, 0.05, 1.0),
        ..default()
    });

    // Primary storage silo
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.4, 3.2, 2.4))),
        MeshMaterial3d(silo_mat.clone()),
        Transform::from_translation(pos + Vec3::Y * 1.6),
        MachineMarker {
            machine_id: id,
            machine_type: MachineVisualType::StorageBay,
        },
        EnvironmentEntity,
    ));

    // Hazard stripe band
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.46, 0.25, 2.46))),
        MeshMaterial3d(hazard_mat),
        Transform::from_translation(pos + Vec3::Y * 0.6),
        EnvironmentEntity,
    ));

    // Reinforced silo lid
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.5, 0.18, 2.5))),
        MeshMaterial3d(silo_mat),
        Transform::from_translation(pos + Vec3::Y * 3.3),
        EnvironmentEntity,
    ));
}

/// Raycast picking system that selects in-world machines upon left mouse click.
pub fn machine_pick_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    windows: Query<&Window>,
    machine_query: Query<(&MachineMarker, &GlobalTransform)>,
    mut game: ResMut<crate::GameSession>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_query.single() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return;
    };

    let dir = ray.direction.as_vec3();
    let mut closest: Option<(f32, u32)> = None;

    for (marker, machine_transform) in &machine_query {
        let machine_pos = machine_transform.translation();
        let to_machine = machine_pos - ray.origin;
        let t = to_machine.dot(dir);
        if t > 0.0 {
            let closest_point = ray.origin + dir * t;
            let dist = (closest_point - machine_pos).length();
            if dist < 2.2 && closest.is_none_or(|(ct, _)| t < ct) {
                closest = Some((t, marker.machine_id));
            }
        }
    }

    if let Some((_, id)) = closest {
        game.session.inspector.select_machine(id);
    }
}
