//! Industrial Aethelgard 3D environment — Dropship Bay floor, platform, walls, and lighting.

use bevy::prelude::*;

/// Marker component for 3D environment elements to enable lifecycle cleanup.
#[derive(Component)]
pub struct EnvironmentEntity;

/// Spawns the Dropship Bay environment: graphite floor, platform, bulkhead walls, and atmospheric lighting.
pub fn spawn_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor — dark graphite industrial plating
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11),
        metallic: 0.35,
        perceptual_roughness: 0.85,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(35.0, 35.0)))),
        MeshMaterial3d(floor_mat.clone()),
        Transform::default(),
        EnvironmentEntity,
    ));

    // Raised reinforced platform where machines operate
    let platform_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.16),
        metallic: 0.55,
        perceptual_roughness: 0.65,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.35, 10.0))),
        MeshMaterial3d(platform_mat),
        Transform::from_xyz(0.0, 0.175, 0.0),
        EnvironmentEntity,
    ));

    // Back wall — industrial bulkhead
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.11, 0.13),
        metallic: 0.45,
        perceptual_roughness: 0.75,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(36.0, 10.0, 0.5))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, 5.0, -7.0),
        EnvironmentEntity,
    ));

    // Side bulkheads
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 10.0, 36.0))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(-18.0, 5.0, 0.0),
        EnvironmentEntity,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 10.0, 36.0))),
        MeshMaterial3d(wall_mat),
        Transform::from_xyz(18.0, 5.0, 0.0),
        EnvironmentEntity,
    ));

    // Rust-orange indicator beacons on the platform border
    let beacon_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.38, 0.17),
        emissive: LinearRgba::new(0.92, 0.38, 0.17, 2.5),
        ..default()
    });

    for i in 0..5 {
        let x = -8.0 + (i as f32) * 4.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.4, 0.12, 0.4))),
            MeshMaterial3d(beacon_mat.clone()),
            Transform::from_xyz(x, 0.40, -4.8),
            EnvironmentEntity,
        ));
    }

    // Cyan scanning strip across the bulkhead wall
    let scan_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.82, 0.95),
        emissive: LinearRgba::new(0.12, 0.82, 0.95, 2.0),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(18.0, 0.08, 0.08))),
        MeshMaterial3d(scan_mat),
        Transform::from_xyz(0.0, 2.5, -6.7),
        EnvironmentEntity,
    ));

    // Overhead industrial point lights
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.70, 0.35),
            intensity: 45_000.0,
            range: 26.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 8.5, 1.0),
        EnvironmentEntity,
    ));

    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.85, 1.0),
            intensity: 22_000.0,
            range: 16.0,
            ..default()
        },
        Transform::from_xyz(-6.0, 4.5, 3.0),
        EnvironmentEntity,
    ));
}

/// Cleans up environment entities upon exiting in-game state.
pub fn cleanup_environment(mut commands: Commands) {
    commands.despawn_all::<With<EnvironmentEntity>>();
}
