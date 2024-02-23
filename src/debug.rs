use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::system::Query;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Resource;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::helper::format_vec3f;
use crate::CameraControl;
use crate::MovableBall;
use crate::MovableCube;
use crate::Terrain;

pub struct DebugTextPlugin;

impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DebugTextState>()
            .add_systems(Update, debug_ui_system);
    }
}

#[derive(Resource)]
pub struct DebugTextState {
    worldinspector: bool,
}

impl Default for DebugTextState {
    fn default() -> Self {
        Self {
            worldinspector: false,
        }
    }
}

// https://whoisryosuke.com/blog/2023/getting-started-with-egui-in-rust
fn debug_ui_system(mut contexts: EguiContexts,
    mut text_state: ResMut<DebugTextState>,
    terrain: Res<Terrain>,
    ball_query: Query<(&Transform, &Velocity), (With<MovableBall>,Without<MovableCube>,Without<CameraControl>)>,) {
    egui::Window::new("Debug output").show(contexts.ctx_mut(), |ui| {
        let (ball_transform, velocity) = ball_query.single();
        ui.horizontal(|ui| {
            ui.label(format!("LOC:{}", format_vec3f(ball_transform.translation)));
            ui.label(format!("VEL:{}", format_vec3f(velocity.linvel)));
            ui.label(format!("ROT:{}", format_vec3f(ball_transform.rotation.xyz())));
            let (width, depth) = terrain.mesh_size;
            let x = (ball_transform.translation.x / terrain.size as f32) as i32 % width as i32;
            let y = (ball_transform.translation.z / terrain.size as f32) as i32 % depth as i32;
            ui.label(format!("MESH:[{x:2.0}]-[{y:2.0}]"));//({:>8.3},{:>8.3},{:>8.3})"
            });
        
        ui.separator();
        ui.checkbox(&mut text_state.worldinspector, "WorldInspector")
        // TODO: enable/disable worldinspector
    });
}
