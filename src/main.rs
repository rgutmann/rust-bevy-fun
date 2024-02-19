use std::f32::consts::TAU;
use bevy::input::mouse::{MouseMotion, MouseButton};
//use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use debug::DebugTextPlugin;
use mesh::{create_mesh, load_elevation_map};
use rand::prelude::*;
use bevy::prelude::*;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy_rapier3d::prelude::*;
//use bevy::window::{CursorGrabMode, Cursor};
//use bevy_rapier3d::render::RapierDebugRenderPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use helper::{ SimpleTween, VelocityTween };

mod helper;
mod mesh;
mod debug;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cube-Mania!    ---> use WASD to move, mouse to rotate, left MB to power up, Space/C to elevate/dive, Esc to quit <---".to_string(),
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
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        //.add_plugin(WireframePlugin)
        .add_systems(Startup, setup)
        .add_plugins(DebugTextPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(Terrain::default())
        .add_systems(Update, user_actions)
        .add_systems(Update, cube_orbit_movement)
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
    const INITIAL_POSITION:Transform = Transform::from_xyz(40.0, 10.0, 30.0);
    const DEATH_HEIGHT:f32 = -10.0;
    const MAX_MOVEMENT_SPEED:f32 = 12.0;
    const INC_MOVEMENT_SPEED:f32 = 20.0; // times delta_seconds
    const SHIFT_MOVEMENT_MULTIPLIER:f32 = 3.0;
    const JUMP_SPEED:f32 = 5.0;
    const FLY_MODE:bool = true;
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

#[derive(Resource)]
struct Terrain {
    size: f64,
    intensity: f32,
}
impl Terrain {
    const DEFAULT_SIZE:f64 = 200.0;
    const DEFAULT_INTENSITY:f32 = 4.0;
    fn _reset(&mut self) {
        self.size = Terrain::DEFAULT_SIZE;
        self.intensity = Terrain::DEFAULT_INTENSITY;
    }
}
impl Default for Terrain {
    fn default() -> Self {
        Terrain { 
            size: Terrain::DEFAULT_SIZE,
            intensity: Terrain::DEFAULT_INTENSITY,
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
    asset_server: Res<AssetServer>,
    terrain: Res<Terrain>,
    //seed: Res<Seed>,
) {
    
    // // plane
    // let _plane_entity = commands.spawn(PbrBundle {
    //         mesh: meshes.add(shape::Box::new(terrain.size as f32, 0.1, terrain.size as f32).into()),
    //         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //         transform: Transform::from_xyz(0.0, -0.1, 0.0),
    //         ..default()
    //     })
    //     .insert(Name::new("Plane"))
    //     .insert(Collider::cuboid(terrain.size as f32/2.0, 0.05, terrain.size as f32/2.0))
    //     .id();
    
    // terrain

    // // randomly generated noisemap
    // let width: usize = 512;
    // let depth: usize = 512;
    // let frequency = 0.1;
    // let lacunarity = 2.0;
    // let octaves = 6;
    // let create_file = true;
    // let noisemap = generate_noisemap(terrain.size, width, depth, frequency, lacunarity, octaves, create_file);
    // let elevation_map: Handle<Image> = asset_server.load("fbm.png").into();
    // let map = load_elevation_map( "example_images/fbm.png", 2.0);

    // staigermanus dogwaffle terrain3 map https://www.renderosity.com/freestuff/items/77673
    let color_map: Handle<Image> = asset_server.load("dogwaffle-terrain3/dogwaffle-terrain3-colr.png").into();
    let map = load_elevation_map( "assets/dogwaffle-terrain3/dogwaffle-terrain3-elev.png", 4.0);
    let (width, depth) = map.size();

    let mesh = create_mesh(terrain.size, width, depth, map, terrain.intensity);
    commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(color_map.into()),
            transform: Transform::from_xyz(0.0, -0., 0.0),
            ..Default::default()
        })
        //.insert(Collider::from_bevy_mesh(&mesh,ComputedColliderShape::TriMesh))
        //.insert(Wireframe)
        .insert(Name::new("Terrain"));


    // Create the ball
    let ball_entity = commands
        .spawn(RigidBody::Dynamic)
        .insert(Name::new("Ball"))
        .insert((PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: MovableBall::RADIUS, sectors: 36, stacks: 36 })),
                material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
                transform: MovableBall::INITIAL_POSITION,
                ..default()
             }, 
             MovableBall::default(),
        ))
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert( if !MovableBall::FLY_MODE {GravityScale(1.0)} else {GravityScale(0.0)} )
        .insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z)
        .insert(Collider::ball(MovableBall::RADIUS))
        .insert(ColliderDebugColor(Color::WHITE))
        .insert(Restitution::coefficient(0.7))
        .id();

    // Create cubes as childs of the ball
    let cube_count = 50;
    let mut rng = rand::thread_rng();
    for i in 1..=cube_count {
        let mut position = Transform::from_xyz(rng.gen_range(1.0..2.0),rng.gen_range(-0.25..0.25),0.0);
        position.translate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, -TAU / cube_count as f32 * i as f32));

        let cube = commands.spawn((PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.12 })),
                material: materials.add(calc_rainbow_color(0, cube_count, i-1).into()),
                transform: position,
                ..default()
            }, 
            MovableCube,
        )).id();

        commands.entity(ball_entity).push_children(&[cube]);
    }

    // Create light and attach to ball
    let light_entity = commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 10000.0,
            range: 200.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
        })
        .insert(Name::new("Light"))
        .id();
    commands.entity(ball_entity).push_children(&[light_entity]);
    
    // Attach camera to ball
    let camera_entity = commands.spawn((Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.5, 5.0).looking_at(Vec3::new(1.0, 1.25, 0.0), Vec3::Y),
            ..default()
        }, 
        CameraControl,
        ))
        .insert(Name::new("Camera"))
        .id();
    commands.entity(ball_entity).push_children(&[camera_entity]);

}

fn user_actions(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut ev_motion: EventReader<MouseMotion>,
    res_buttons:  Res<Input<MouseButton>>,
    mut ball_query: Query<(&mut MovableBall, &mut Transform, &mut Velocity), (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>,
    camera_query: Query<&Transform, With<CameraControl>>
) {
    let (mut ball, mut ball_transform, mut velocity) = ball_query.single_mut();
    let camera_transform = camera_query.single();
    let camera_rotation_misalignment = Vec3::Z.angle_between(Vec3::new( 
            camera_transform.translation.x, 0.0, camera_transform.translation.z 
        ));
        
    if ball_transform.translation.y < MovableBall::DEATH_HEIGHT {
        ball_transform.translation = MovableBall::INITIAL_POSITION.translation;
    }

    //
    // input based movement, but only when touching ground
    if MovableBall::FLY_MODE || ball_transform.translation.y <= MovableBall::RADIUS {

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
            movement.rotate_around(Vec3::ZERO, Quat::from_rotation_y(0.5*camera_rotation_misalignment));
            ball.velocity.add_velocity(time.delta_seconds(), movement.translation);
        } else {
            ball.velocity.slowdown(time.delta_seconds());
        }

        if !MovableBall::FLY_MODE {
            if input.pressed(KeyCode::Space) {
                velocity.linvel.y = MovableBall::JUMP_SPEED;
            }
            if input.pressed(KeyCode::C) {
                velocity.linvel.y = -MovableBall::JUMP_SPEED;
            }
        } else {
            if input.pressed(KeyCode::Space) {
                velocity.linvel.y = MovableBall::JUMP_SPEED;
            } else if input.pressed(KeyCode::C) {
                velocity.linvel.y = -MovableBall::JUMP_SPEED;
            } else { 
                velocity.linvel.y = 0.
            }
        }
    } else {
        // airborne... current movement is locked
    }
    // apply horizontal velocity, but don't change vertical velocity
    let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) { MovableBall::SHIFT_MOVEMENT_MULTIPLIER } else { 1.0 };
    let cur_velicity = (*ball.velocity.current_velocity())*speed_multiplier + Vec3::new(0.0, velocity.linvel.y, 0.0);
    velocity.linvel = cur_velicity;

    //
    // ball rotation
    // accumulate "ball/camera" rotation from mouse movement
    let mut rotation_move = Vec2::ZERO;
    for ev in ev_motion.read() {
        rotation_move += ev.delta;
    }
    // rotate ball accordingly
    ball_transform.rotate_y(-rotation_move.x * 0.01);

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
