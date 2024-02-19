use std::f32::consts::TAU;
use bevy::input::mouse::{MouseMotion, MouseButton};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy_infinite_grid::{InfiniteGridPlugin, InfiniteGridBundle, InfiniteGrid};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use mesh::create_mesh;
use rand::prelude::*;
use bevy::prelude::*;
use bevy::diagnostic::LogDiagnosticsPlugin;
//use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy_rapier3d::prelude::*;
//use bevy::window::{CursorGrabMode, Cursor};
//use bevy_rapier3d::render::RapierDebugRenderPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use helper::{ SimpleTween, VelocityTween, format_vec3f };

mod helper;
mod mesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cube-Mania!    ---> use WASD to move, mouse to rotate, left MB to power up, Space to jump, Esc to quit <---".to_string(),
                // cursor: { 
                //     let mut cursor = Cursor::default(); 
                //     cursor.visible = false; 
                //     cursor.grab_mode = CursorGrabMode::Locked;
                //     cursor 
                // },
                ..default()
            }),
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InfiniteGridPlugin)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(user_actions)
        .add_system(cube_orbit_movement)
        .run();
}


#[derive(Component)]
struct MovableCube;

#[derive(Component,Debug)]
struct MovableBall {
    velocity:VelocityTween,
    orbit_speed:SimpleTween,
}
impl MovableBall {
    const RADIUS:f32 = 0.5;
    const MAX_MOVEMENT_SPEED:f32 = 0.1;
    const INC_MOVEMENT_SPEED:f32 = 0.5; // times delta_seconds
    const JUMP_SPEED:f32 = 5.0;
    const MIN_ORBIT_SPEED:f32 = 50.0;
    const MAX_ORBIT_SPEED:f32 = 300.0;
    const INC_ORBIT_SPEED:f32 = 5.0;
}
impl Default for MovableBall {
    fn default() -> Self {
        MovableBall { 
            velocity: VelocityTween::new( 
                Vec3::ZERO, 
                0.0, 
                MovableBall::MAX_MOVEMENT_SPEED, 
                MovableBall::INC_MOVEMENT_SPEED, 
            ),
            orbit_speed: SimpleTween::new( 
                MovableBall::MIN_ORBIT_SPEED, 
                MovableBall::MIN_ORBIT_SPEED, 
                MovableBall::MAX_ORBIT_SPEED, 
                MovableBall::INC_ORBIT_SPEED, 
            )
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
    //asset_server: Res<AssetServer>,
    //seed: Res<Seed>,
) {
    // infinite grid
    commands.spawn(InfiniteGridBundle {
        grid: InfiniteGrid {
            // shadow_color: None,
            ..Default::default()
        },
        ..Default::default()
    });

    // plane
    let plane_size = 50.0;
    let plane_entity = commands.spawn(PbrBundle {
            mesh: meshes.add(shape::Box::new(plane_size, 0.1, plane_size).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(plane_size/2.0, 0.05, plane_size/2.0))
        .id();
    
    // terrain
    let extent: f64 = plane_size as f64;
    let intensity = 1.0;
    let width: usize = 128;
    let depth: usize = 128;
    let mesh = create_mesh(extent, intensity, width, depth);
    let terrain = commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.3, 0.0),
            ..Default::default()
        })
        //.insert(Collider::from_bevy_mesh(&mesh,ComputedColliderShape::TriMesh))
        .insert(Wireframe)
        .id();
    commands.entity(plane_entity).push_children(&[terrain]);


    // Create the bouncing ball
    let ball_entity = commands
        .spawn(RigidBody::Dynamic)
        .insert((PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: MovableBall::RADIUS, sectors: 36, stacks: 36 })),
                material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
                transform: Transform::from_xyz(0.0, 4.0, 0.0),
                ..default()
             }, 
             MovableBall::default(),
        ))
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert(Collider::ball(MovableBall::RADIUS))
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
    mut ball_query: Query<(&mut MovableBall, &mut Transform, &mut Velocity), (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>
) {
    let (mut ball, mut ball_transform, mut velocity) = ball_query.single_mut();
    
    println!("LOC:{} - VEL:{} - BVEL:{}", format_vec3f(ball_transform.translation), format_vec3f(velocity.linvel), format_vec3f(*ball.velocity.current_velocity()));

    //
    // input based movement, but only when touching ground
    if ball_transform.translation.y <= MovableBall::RADIUS {

        // accumulate key-based movement
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
        if direction.length() >= 1.0 {
            let ball_rotation = ball_transform.rotation;
            let mut movement = Transform::from_translation(direction);
            movement.rotate_around(Vec3::ZERO, ball_rotation);
            ball.velocity.add_velocity(time.delta_seconds(), movement.translation);
        } else {
            ball.velocity.slowdown(time.delta_seconds());
        }

        if input.pressed(KeyCode::Space) {
            // set vertical velocity to fixed jump speed
            velocity.linvel.y = MovableBall::JUMP_SPEED;
    }
    } else {
        // airborne... current movement is locked
    }
    ball_transform.translation += *ball.velocity.current_velocity();

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
        ball.orbit_speed.increase_once();
    } else {
        ball.orbit_speed.decrease_once();
    }
}


fn cube_orbit_movement(
    time: Res<Time>,
    mut cube_query: Query<&mut Transform, With<MovableCube>>,
    ball_query: Query<&MovableBall, (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>
) {
    let ball = ball_query.get_single().unwrap();
    let rotation_speed = ball.orbit_speed.current_value();

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
