use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashSet;

use crate::GRID_SIZE;

pub const START_IID: &str = "4d96be30-5e50-11f0-a5b5-af79102484b9";
const LEVEL_IIDS: [&str; 1] = [
    "42c222c0-5e50-11f0-b1e7-dbe8e4236e84",
    // "0b33a750-5e50-11f0-b1e7-25e2d47bc36d",
    // "34d7e6c0-5e50-11f0-b1e7-65faa10719f2",
];

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cache_wall_locations)
            .add_systems(Startup, setup_world)
            .add_systems(Startup, load_level)
            .register_ldtk_int_cell::<WallBundle>(1);
    }
}

#[derive(Component)]
struct Tile {
    pub grid_coords: GridCoords,
}

#[derive(Default, Component)]
struct Wall;
#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Default, Debug, Clone, Resource)]
pub struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    pub fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        // grid_coords.x < 0
        //     || grid_coords.y < 0
        //     || grid_coords.x >= self.level_width
        //     || grid_coords.y >= self.level_height
        self.wall_locations.contains(grid_coords)
    }
}

// only loads the initial level
fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let level_set = LevelSet::from_iids([START_IID]);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("train.ldtk").into(),
        level_set,
        // transform: Transform::from_xyz(-256., -144., 0.),
        ..Default::default()
    });
    // setup walls
    commands.init_resource::<LevelWalls>();
}

// loads levels, with the avaliable world bundles when requested
// TODO: make a resource to handle this
fn load_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    let level_set = LevelSet::from_iids(LEVEL_IIDS);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("train.ldtk").into(),
        level_set,
        transform: Transform::from_xyz(0., 256., 0.),
        ..Default::default()
    });
}

// TODO: cant do multiple worlds (yet)
fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            for handle in ldtk_project_entities {
                let ldtk_project = ldtk_project_assets
                    .get(handle)
                    .expect("LdtkProject should be loaded when level is spawned");
                let level = ldtk_project
                    .get_raw_level_by_iid(level_iid.get())
                    .expect("spawned level should exist in project");

                let wall_locations = walls.iter().copied().collect();

                let new_level_walls = LevelWalls {
                    wall_locations,
                    level_width: level.px_wid / GRID_SIZE,
                    level_height: level.px_hei / GRID_SIZE,
                };

                *level_walls = new_level_walls.clone();
                println!("{:?}", new_level_walls.wall_locations.len());
            }
        }
    }
    Ok(())
}
