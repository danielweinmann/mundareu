use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::RunOptions;

const SCREENSHOT_DELAY_IN_SECONDS: f32 = 2.0;
const FRAME_RATE_LOG_INTERVAL_IN_SECONDS: f32 = 2.0;

#[derive(Resource)]
struct ProofTimers {
    screenshot: Option<Timer>,
    exit: Option<Timer>,
    frame_rate_log: Option<Timer>,
}

pub fn plugin(app: &mut App) {
    let options = app.world().resource::<RunOptions>().clone();
    app.insert_resource(ProofTimers {
        screenshot: options
            .screenshot_path
            .is_some()
            .then(|| Timer::from_seconds(SCREENSHOT_DELAY_IN_SECONDS, TimerMode::Once)),
        exit: options
            .exit_after_seconds
            .map(|seconds| Timer::from_seconds(seconds, TimerMode::Once)),
        frame_rate_log: options
            .log_frame_rate
            .then(|| Timer::from_seconds(FRAME_RATE_LOG_INTERVAL_IN_SECONDS, TimerMode::Repeating)),
    })
    .add_systems(Update, (capture_screenshot, exit_when_due, log_frame_rate));
}

fn capture_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    options: Res<RunOptions>,
    mut timers: ResMut<ProofTimers>,
) {
    let Some(timer) = &mut timers.screenshot else {
        return;
    };
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    if let Some(path) = &options.screenshot_path {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
    }
}

fn exit_when_due(
    time: Res<Time>,
    mut timers: ResMut<ProofTimers>,
    mut exits: MessageWriter<AppExit>,
) {
    let Some(timer) = &mut timers.exit else {
        return;
    };
    if timer.tick(time.delta()).just_finished() {
        exits.write(AppExit::Success);
    }
}

fn log_frame_rate(
    time: Res<Time>,
    mut timers: ResMut<ProofTimers>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Some(timer) = &mut timers.frame_rate_log else {
        return;
    };
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    if let Some(frames_per_second) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
    {
        info!("{frames_per_second:.1} frames per second");
    }
}
