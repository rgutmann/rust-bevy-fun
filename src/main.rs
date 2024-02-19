use std::f32::consts::TAU;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use rand::prelude::*;
use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_rapier3d::prelude::*;
//use bevy_rapier3d::render::RapierDebugRenderPlugin;


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
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(ball_movement)
        .add_system(cube_movement)
        .run();
}

#[derive(Component)]
struct MovableCube;

#[derive(Component)]
struct MovableBall;

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


    // Create the bouncing ball
    let ball_entity = commands
        .spawn(RigidBody::Dynamic)
        .insert((PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 36, stacks: 36 })),
                material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
                transform: Transform::from_xyz(0.0, 4.0, 0.0),
                ..default()
             }, 
             MovableBall,
        ))
        .insert(Collider::ball(0.5))
        .insert(ColliderDebugColor(Color::WHITE))
        .insert(Restitution::coefficient(0.7))
        .id();

    // Create cubes as childs
    let cube_count = 50;
    let mut rng = rand::thread_rng();
    for i in 1..=cube_count {
        let mut position = Transform::from_xyz(rng.gen_range((plane_size*0.2)..(plane_size*0.4)),rng.gen_range(-0.25..0.25),0.0);
        position.translate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, -TAU / cube_count as f32 * i as f32));

        let child = commands.spawn((PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
                material: materials.add(calc_rainbow_color(0, cube_count, i-1).into()),
                transform: position,
                ..default()
            }, 
            MovableCube,
        )).id();

        commands.entity(ball_entity).push_children(&[child]);
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
    
    // Attach camera to ball
    let camera_entity = commands.spawn((Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::new(1.0, 1.25, 0.0), Vec3::Y),
            ..default()
        }, 
        CameraControl,
    )).id();
    commands.entity(ball_entity).push_children(&[camera_entity]);

}


fn ball_movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ball_query: Query<&mut Transform, (With<MovableBall>,Without<CameraControl>)>
) {
    // determine key-based movement
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::W) {
        direction.z -= 1.0;
    }
    if input.pressed(KeyCode::S) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    // key-based movement
    let mut ball_transform = ball_query.single_mut();
    let ball_rotation = ball_transform.rotation;
    let mut movement = Transform::from_translation(time.delta_seconds() * 2.0 * direction);
    println!("move({:?}) brot({:?})", movement, ball_rotation);
    movement.rotate_around(Vec3::ZERO, ball_rotation);
    println!("  -> move+({:?})", movement);
    ball_transform.translation += movement.translation;

    // determine rotation
    let mut rotation_move = Vec2::ZERO;
    for ev in ev_motion.iter() {
        rotation_move += ev.delta;
    }
    // rotate around center
    ball_transform.rotate_y(rotation_move.x * 0.01);
}


fn cube_movement(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MovableCube>>,
) {
    for mut transform in &mut query {
        let gpos_start = transform.translation;
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
