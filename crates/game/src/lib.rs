use std::path::PathBuf;

use bevy::prelude::*;

mod avatar_plugin;
mod building_plugin;
mod hud_plugin;
mod input_plugin;
mod locale;
mod persistence_plugin;
mod proof_plugin;
mod world_plugin;

pub use bevy::app::AppExit;

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct RunOptions {
    pub screenshot_path: Option<PathBuf>,
    pub exit_after_seconds: Option<f32>,
    pub log_frame_rate: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameSystems {
    Building,
    Terrain,
    Avatar,
    Camera,
    Hud,
}

pub fn run() -> AppExit {
    run_with(RunOptions::default())
}

pub fn run_with(options: RunOptions) -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(primary_window()),
        ..default()
    }));
    let saved_game = persistence_plugin::load_or_generate();
    app.insert_resource(options)
        .insert_resource(world_plugin::Terrain(saved_game.terrain))
        .insert_resource(avatar_plugin::StartingPlayer(saved_game.player))
        .configure_sets(
            Update,
            (
                GameSystems::Building,
                GameSystems::Terrain,
                GameSystems::Avatar,
                GameSystems::Camera,
                GameSystems::Hud,
            )
                .chain(),
        )
        .add_plugins((
            world_plugin::plugin,
            avatar_plugin::plugin,
            input_plugin::plugin,
            building_plugin::plugin,
            hud_plugin::plugin,
            persistence_plugin::plugin,
            proof_plugin::plugin,
        ));
    #[cfg(target_os = "ios")]
    app.insert_resource(bevy::winit::WinitSettings::mobile());
    app.run()
}

#[cfg(target_os = "ios")]
fn primary_window() -> Window {
    Window {
        title: locale::WINDOW_TITLE.to_string(),
        mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        resizable: false,
        recognize_rotation_gesture: true,
        prefers_home_indicator_hidden: true,
        prefers_status_bar_hidden: true,
        preferred_screen_edges_deferring_system_gestures: bevy::window::ScreenEdge::Bottom,
        ..default()
    }
}

#[cfg(not(target_os = "ios"))]
fn primary_window() -> Window {
    Window {
        title: locale::WINDOW_TITLE.to_string(),
        resolution: bevy::window::WindowResolution::new(1366, 1024),
        ..default()
    }
}
