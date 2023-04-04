//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::f32::consts::TAU;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement)
        .run();
}

#[derive(Component)]
struct Movable;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    println!("{:?}", Mesh::from(shape::Plane::from_size(5.0)));
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cubes
    for i in 1..11 {
        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz((i as f32 - 5.0) / 2.0 - 0.25, 0.5, 0.0),
            ..default()
        },
        Movable,));
    }
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Movable>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }

        transform.translation += time.delta_seconds() * 2.0 * direction;

        // rotate around center
        let gpos_start = transform.translation;
        transform.rotate_around(Vec3::new(0f32,0f32,0f32), Quat::from_rotation_y(time.delta_seconds() * 0.5));
        let gpos_end = transform.translation;

        // TODO: rotate object in direction of movement
        let movement = gpos_end - gpos_start;
        let stable_vec = Vec3::from_array([0.0,-1.0,0.0]);
        let rotation_vec = movement.cross(stable_vec);
        //transform.rotate_local_axis(movement, TAU * time.delta_seconds() * 0.5);
        transform.rotate_x(rotation_vec.x * TAU * time.delta_seconds() * 50.0);
        transform.rotate_y(rotation_vec.y * TAU * time.delta_seconds() * 50.0);
        transform.rotate_z(rotation_vec.z * TAU * time.delta_seconds() * 50.0);

    }
}