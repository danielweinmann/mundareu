use std::io::Write;
use std::path::{Path, PathBuf};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{AppLifecycle, ExitSystems, WindowCloseRequested};
use mundareu_world::{PlayerState, decode, encode, generate_flat_hills};

use crate::avatar_plugin::{Avatar, OrbitCamera};
use crate::world_plugin::{BlockChanged, Terrain};

const AUTOSAVE_INTERVAL_IN_SECONDS: f32 = 15.0;
const WORLD_RADIUS_IN_CHUNKS: i32 = 5;
const WORLD_SEED: u64 = 2026;
const SAVE_FILE_NAME: &str = "world.mundareu";

pub struct SavedGame {
    pub terrain: mundareu_world::World,
    pub player: Option<PlayerState>,
}

#[derive(Resource)]
struct SavePath(Option<PathBuf>);

#[derive(Resource)]
struct Autosave {
    timer: Timer,
    dirty: bool,
}

#[derive(SystemParam)]
struct SaveTarget<'w, 's> {
    path: Res<'w, SavePath>,
    terrain: Res<'w, Terrain>,
    avatar: Single<'w, 's, &'static Avatar>,
    camera: Single<'w, 's, &'static OrbitCamera>,
}

impl SaveTarget<'_, '_> {
    fn write(&self) {
        let Some(path) = &self.path.0 else {
            return;
        };
        let player = PlayerState {
            position: self.avatar.body.position,
            yaw: self.camera.yaw,
        };
        match write_bytes(path, &encode(&self.terrain.0, player)) {
            Ok(()) => info!("saved the world to {}", path.display()),
            Err(error) => error!("could not save the world to {}: {error}", path.display()),
        }
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(SavePath(save_path()))
        .insert_resource(Autosave {
            timer: Timer::from_seconds(AUTOSAVE_INTERVAL_IN_SECONDS, TimerMode::Repeating),
            dirty: false,
        })
        .add_systems(
            Update,
            (mark_dirty_on_block_change, autosave_periodically).chain(),
        )
        .add_systems(Last, save_on_suspend_or_exit.after(ExitSystems));
}

pub fn load_or_generate() -> SavedGame {
    let Some(path) = save_path() else {
        warn!("no data directory is available, so the world will not be saved");
        return fresh_game();
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            info!(
                "no saved world at {}, generating a fresh one",
                path.display()
            );
            return fresh_game();
        }
        Err(error) => {
            return set_aside_and_generate(
                &path,
                format!(
                    "could not read the saved world at {}: {error}",
                    path.display()
                ),
            );
        }
    };
    match decode(&bytes) {
        Ok((terrain, player)) => {
            info!("loaded the saved world from {}", path.display());
            SavedGame {
                terrain,
                player: Some(player),
            }
        }
        Err(error) => set_aside_and_generate(
            &path,
            format!(
                "could not decode the saved world at {}: {error}",
                path.display()
            ),
        ),
    }
}

fn set_aside_and_generate(path: &Path, problem: String) -> SavedGame {
    let unreadable_path = unreadable_save_path(path);
    match std::fs::rename(path, &unreadable_path) {
        Ok(()) => warn!(
            "{problem}; moved it to {} and generating a fresh world",
            unreadable_path.display()
        ),
        Err(error) => warn!(
            "{problem}; could not move it to {}: {error}; generating a fresh world that will replace it on the next save",
            unreadable_path.display()
        ),
    }
    fresh_game()
}

fn fresh_game() -> SavedGame {
    SavedGame {
        terrain: generate_flat_hills(WORLD_RADIUS_IN_CHUNKS, WORLD_SEED),
        player: None,
    }
}

#[cfg(target_os = "ios")]
fn save_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Documents").join(SAVE_FILE_NAME))
}

#[cfg(not(target_os = "ios"))]
fn save_path() -> Option<PathBuf> {
    let base_directories = directories::BaseDirs::new()?;
    Some(
        base_directories
            .data_dir()
            .join(crate::locale::WINDOW_TITLE)
            .join(SAVE_FILE_NAME),
    )
}

fn mark_dirty_on_block_change(
    mut block_changes: MessageReader<BlockChanged>,
    mut autosave: ResMut<Autosave>,
) {
    if block_changes.read().next().is_some() {
        autosave.dirty = true;
    }
}

fn autosave_periodically(time: Res<Time>, mut autosave: ResMut<Autosave>, target: SaveTarget) {
    if !autosave.timer.tick(time.delta()).just_finished() || !autosave.dirty {
        return;
    }
    target.write();
    autosave.dirty = false;
}

fn save_on_suspend_or_exit(
    mut lifecycle_messages: MessageReader<AppLifecycle>,
    mut close_requests: MessageReader<WindowCloseRequested>,
    mut exits: MessageReader<AppExit>,
    mut autosave: ResMut<Autosave>,
    target: SaveTarget,
) {
    let suspended = lifecycle_messages
        .read()
        .any(|lifecycle| *lifecycle == AppLifecycle::Suspended);
    let leaving = close_requests.read().next().is_some() || exits.read().next().is_some();
    if suspended || leaving {
        target.write();
        autosave.dirty = false;
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let partial_path = partial_save_path(path);
    let mut partial_file = std::fs::File::create(&partial_path)?;
    partial_file.write_all(bytes)?;
    partial_file.sync_all()?;
    std::fs::rename(&partial_path, path)
}

fn partial_save_path(path: &Path) -> PathBuf {
    path_with_suffix(path, ".partial")
}

fn unreadable_save_path(path: &Path) -> PathBuf {
    path_with_suffix(path, ".unreadable")
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut path_with_suffix = path.as_os_str().to_owned();
    path_with_suffix.push(suffix);
    PathBuf::from(path_with_suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_save_is_written_beside_the_world_file_before_replacing_it() {
        assert_eq!(
            partial_save_path(Path::new("/saves/world.mundareu")),
            PathBuf::from("/saves/world.mundareu.partial")
        );
    }

    #[test]
    fn an_unreadable_world_is_kept_beside_the_world_file() {
        assert_eq!(
            unreadable_save_path(Path::new("/saves/world.mundareu")),
            PathBuf::from("/saves/world.mundareu.unreadable")
        );
    }
}
