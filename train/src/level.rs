use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;

use crate::GRID_SIZE;
use crate::LEVEL_NUMBER;

pub const START_IID: &str = "4d96be30-5e50-11f0-a5b5-af79102484b9";
const LEVEL_IIDS: [&str; 4] = [
    "42c222c0-5e50-11f0-b1e7-dbe8e4236e84",
    "0b33a750-5e50-11f0-b1e7-25e2d47bc36d",
    "34d7e6c0-5e50-11f0-b1e7-65faa10719f2",
    "4d967011-5e50-11f0-a5b5-b533d53fa5b9",
];

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cache_wall_locations)
            .add_systems(Startup, setup_world)
            .add_systems(Update, load_level)
            .register_ldtk_int_cell::<WallBundle>(1);
    }
}

#[derive(Resource)]
struct WorldHandler {
    loaded_worlds: Vec<WorldData>,
    loading_world: Option<LevelIid>,
    current_state: WorldLoadState,
}
#[derive(Default, Debug, PartialEq)]
enum WorldLoadState {
    #[default]
    Loading,
    Caching,
    Finished,
    Idle,
}
#[derive(Clone, Debug)]
struct WorldData {
    iid: LevelIid,
    height: i32,
    width: i32,
    top: i32,
}

#[derive(Default, Component, Debug)]
struct Wall {
    level_index: usize, // starts at 1 because it defaults to 0 (i want to assign each of them)
}
#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Default, Debug, Clone, Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
}

impl LevelWalls {
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        self.wall_locations.contains(grid_coords)
    }
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // only loads the initial level
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("train.ldtk").into(),
        level_set: LevelSet::from_iids([START_IID]),
        ..Default::default()
    });

    // setup walls
    commands.init_resource::<LevelWalls>();
    // initiating world handler
    commands.insert_resource(WorldHandler {
        loaded_worlds: vec![],
        loading_world: Some(LevelIid::from(START_IID.to_owned())),
        current_state: WorldLoadState::Loading,
    });
}

// loads levels, with the avaliable world bundles when requested
fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut world_handler: ResMut<WorldHandler>,
) {
    if world_handler.current_state != WorldLoadState::Finished {
        return;
    }
    if world_handler.loaded_worlds.len() == LEVEL_NUMBER {
        world_handler.current_state = WorldLoadState::Idle;
        return;
    }

    let mut rng = rand::rng();
    let id = LEVEL_IIDS.choose(&mut rng).unwrap().to_owned();
    let iid = LevelIid::from(id.to_owned());

    world_handler.current_state = WorldLoadState::Loading;
    world_handler.loading_world = Some(iid);

    let prev_top = world_handler.loaded_worlds.last().unwrap().top;
    let level_set = LevelSet::from_iids([id]);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("train.ldtk").into(),
        level_set,
        transform: Transform::from_xyz(0., (prev_top * GRID_SIZE) as f32, 0.),
        ..Default::default()
    });
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut world_handler: ResMut<WorldHandler>,
    mut level_events: EventReader<LevelEvent>,
    mut walls: Query<(&mut GridCoords, &mut Wall)>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            // one handle should have every level data already
            let handle = ldtk_project_entities.iter().nth(0).unwrap();

            let ldtk_project = ldtk_project_assets
                .get(handle)
                .expect("LdtkProject should be loaded when level is spawned");
            let level = ldtk_project
                .get_raw_level_by_iid(level_iid.get())
                .expect("spawned level should exist in project");
            let height = level.px_hei / GRID_SIZE;

            let prev_top = if world_handler.loaded_worlds.len() >= 1 {
                world_handler.loaded_worlds.last().unwrap().top
            } else {
                0
            };

            let world = WorldData {
                iid: level_iid.clone(),
                height,
                width: level.px_wid,
                top: height + prev_top,
            };

            for (mut location, mut wall) in walls.iter_mut() {
                if wall.level_index != 0 {
                    continue;
                }

                wall.level_index = world_handler.loaded_worlds.len() + 1;

                let new_location = location.clone() + GridCoords::new(0, prev_top);
                *location = new_location;
                level_walls.wall_locations.insert(*location);
            }

            world_handler.loaded_worlds.push(world);
            world_handler.current_state = WorldLoadState::Finished;
            world_handler.loading_world = None;
        }
    }
    Ok(())
}
