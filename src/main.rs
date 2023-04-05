use std::f32::consts::TAU;
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use bevy_rapier3d::render::RapierDebugRenderPlugin;
use rand::prelude::*;
use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::LookTransformPlugin;
use smooth_bevy_cameras::controllers::orbit::{OrbitCameraPlugin, OrbitCameraBundle, OrbitCameraController};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cube-Mania!    ---> use cursor keys to move, ctrl+mouse to rotate, scroll to zoom, Esc to quit <---".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(cube_movement)
        .run();
}

#[derive(Component)]
struct MovableCube;

#[derive(Component)]
struct CameraControl;

/// set up 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    let plane_size = 5.0;
    commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(plane_size, 0.1, plane_size).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(plane_size/2.0, 0.05, plane_size/2.0));


    // cubes in a row
    let cube_count = 50;
    let mut rng = rand::thread_rng();
    for i in 1..=cube_count {
        let mut position = Transform::from_xyz(rng.gen_range((plane_size*0.2)..(plane_size*0.4)),rng.gen_range(0.3..0.8),0.0);
        position.translate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, -TAU / cube_count as f32 * i as f32));

        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
            material: materials.add(calc_rainbow_color(0, cube_count, i-1).into()),
            transform: position,
            ..default()
        },
        MovableCube,));
    }

    /* Create the bouncing ball. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 36, stacks: 36 })),
            material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..default()
        })
        .insert(Collider::ball(0.5))
        .insert(ColliderDebugColor(Color::WHITE))
        .insert(Restitution::coefficient(0.7));

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
    commands.spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(-2.0, 2.5, 5.0), 
            Vec3::new(0., 0., 0.),
            Vec3::Y,
    ));
}


fn cube_movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MovableCube>>,
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
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds() * 0.5));
        let gpos_end = transform.translation;

        // rotate object in direction of movement
        let movement = gpos_end - gpos_start;
        let stable_vec = Vec3::Y * -1.0;
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
