use std::f32::consts::TAU;
use bevy::{prelude::*};
use rand::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Cube-Mania!    ---> use cursor keys to move, Esc to quit <---".to_string(),
            ..default()
        }),
        ..default()
    }))
    .add_startup_system(setup)
        .add_system(movement)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Movable;

/// set up 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    let plane_size = 5.0;
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(plane_size).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cubes in a row
    let cube_count = 50;
    let mut rng = rand::thread_rng();
    for i in 1..=cube_count {
        let mut position = Transform::from_xyz(rng.gen_range((plane_size*0.2)..(plane_size*0.4)),rng.gen_range(0.3..0.7),0.0);
        position.translate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, -TAU / cube_count as f32 * i as f32));

        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
            material: materials.add(calc_rainbow_color(0, cube_count, i-1).into()),
            //transform: Transform::from_xyz((i as f32 - 5.0) / 2.0 - 0.25, 0.5, 0.0),
            transform: position,
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
    // determine key-based movement
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::Up) {
        direction.z -= 1.0;
    }
    if input.pressed(KeyCode::Down) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }

    for mut transform in &mut query {
        let gpos_start = transform.translation;
        // key-based movement
        transform.translation += time.delta_seconds() * 2.0 * direction;
        // rotate around center
        transform.rotate_around(Vec3::new(0f32,0f32,0f32), Quat::from_rotation_y(time.delta_seconds() * 0.5));
        let gpos_end = transform.translation;

        // rotate object in direction of movement
        let movement = gpos_end - gpos_start;
        let stable_vec = Vec3::from_array([0.0,-1.0,0.0]);
        let rotation_vec = movement.cross(stable_vec);
        transform.rotate_x(rotation_vec.x * TAU * time.delta_seconds() * 50.0);
        transform.rotate_y(rotation_vec.y * TAU * time.delta_seconds() * 50.0);
        transform.rotate_z(rotation_vec.z * TAU * time.delta_seconds() * 50.0);
    }
}


fn calc_rainbow_color(min :usize, max :usize, val :usize) -> Color {
    let min_hue = 360.0;
    let max_hue = 0.0;
    let cur_percent = (val - min) as f32 / (max - min) as f32;
    Color::hsl(((cur_percent * (max_hue-min_hue) ) + min_hue) as f32, 1.0, 0.5)
}