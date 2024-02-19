use std::f32::consts::TAU;
use bevy::input::mouse::{MouseMotion, MouseButton};
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
                title: "Cube-Mania!    ---> use WASD to move, mouse to rotate, left MB to power up, Esc to quit <---".to_string(),
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
        .add_system(user_actions)
        .add_system(cube_movement)
        .run();
}

#[derive(Component)]
struct MovableCube;

#[derive(Component,Debug)]
struct MovableBall {
    movement_speed:f32,
    min_orbit_speed:f32,
    cur_orbit_speed:f32,
    max_orbit_speed:f32,
    inc_orbit_speed:f32,
}
impl Default for MovableBall {
    fn default() -> Self {
        MovableBall { 
            movement_speed: 2.0,
            min_orbit_speed: 50.0, 
            cur_orbit_speed: 50.0, 
            max_orbit_speed: 200.0,
            inc_orbit_speed: 2.0,
        }
    }
}

#[derive(Component)]
struct CameraControl;

/// set up 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    let plane_size = 10.0;
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
             MovableBall::default(),
        ))
        .insert(Collider::ball(0.5))
        .insert(ColliderDebugColor(Color::WHITE))
        .insert(Restitution::coefficient(0.7))
        .id();

    // Create cubes as childs
    let cube_count = 50;
    let mut rng = rand::thread_rng();
    for i in 1..=cube_count {
        let mut position = Transform::from_xyz(rng.gen_range(1.0..2.0),rng.gen_range(-0.25..0.25),0.0);
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


fn user_actions(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut ev_motion: EventReader<MouseMotion>,
    res_buttons:  Res<Input<MouseButton>>,
    mut ball_query: Query<(&mut MovableBall, &mut Transform), (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>
) {
    let (mut ball, mut ball_transform) = ball_query.single_mut();
    let player_speed = ball.movement_speed;

    //
    // ball movement
    // accumulate key-based movement
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::W) {
        direction.z -= player_speed;
    }
    if input.pressed(KeyCode::S) {
        direction.z += player_speed;
    }
    if input.pressed(KeyCode::A) {
        direction.x -= player_speed;
    }
    if input.pressed(KeyCode::D) {
        direction.x += player_speed;
    }
    //let mut ball_transform = ball_query.single_mut();
    let ball_rotation = ball_transform.rotation;
    let mut movement = Transform::from_translation(time.delta_seconds() * 2.0 * direction);
    movement.rotate_around(Vec3::ZERO, ball_rotation);
    ball_transform.translation += movement.translation;

    //
    // ball rotation
    // accumulate "ball/camera" rotation from mouse movement
    let mut rotation_move = Vec2::ZERO;
    for ev in ev_motion.iter() {
        rotation_move += ev.delta;
    }
    // rotate ball accordingly
    ball_transform.rotate_y(rotation_move.x * 0.01);

    //
    // cubes acceleration
    if res_buttons.pressed(MouseButton::Left) {
        ball.cur_orbit_speed = (ball.cur_orbit_speed + ball.inc_orbit_speed).min(ball.max_orbit_speed);
    } else {
        ball.cur_orbit_speed = (ball.cur_orbit_speed - ball.inc_orbit_speed).max(ball.min_orbit_speed);
    }
}


fn cube_movement(
    time: Res<Time>,
    mut cube_query: Query<&mut Transform, With<MovableCube>>,
    ball_query: Query<&MovableBall, (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>
) {
    let ball = ball_query.get_single().unwrap();
    let rotation_speed = ball.cur_orbit_speed;

    for mut transform in &mut cube_query {
        let gpos_start = transform.translation;
        // rotate around center
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(time.delta_seconds() * rotation_speed / 100.0));
        let gpos_end = transform.translation;

        // rotate object in direction of movement
        let movement = gpos_end - gpos_start;
        let stable_vec = Vec3::Y * -1.0;
        let rotation_vec = movement.cross(stable_vec);
        transform.rotate_x(rotation_vec.x * TAU * time.delta_seconds() * rotation_speed / 2.0);
        transform.rotate_y(rotation_vec.y * TAU * time.delta_seconds() * rotation_speed / 2.0);
        transform.rotate_z(rotation_vec.z * TAU * time.delta_seconds() * rotation_speed / 2.0);
    }
}

fn calc_rainbow_color(min :usize, max :usize, val :usize) -> Color {
    let min_hue = 360.0;
    let max_hue = 0.0;
    let cur_percent = (val - min) as f32 / (max - min) as f32;
    Color::hsl(((cur_percent * (max_hue-min_hue) ) + min_hue) as f32, 1.0, 0.5)
}
