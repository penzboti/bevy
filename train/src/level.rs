use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashSet;

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

#[derive(Resource)]
struct WorldHandler {
    loaded_worlds: Vec<LevelIid>, // TODO: separate loaded and cached
}

#[derive(Default, Component)]
struct Wall {
    level: LevelIid, // an idea, unimplemented
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
        // loaded_worlds: vec![LevelIid::from(START_IID.to_owned())]
    });
}

// TODO: loads levels, with the avaliable world bundles when requested
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
    mut world_handler: ResMut<WorldHandler>,
    mut level_events: EventReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            // maybe i could the levels one by one and then cache the walls separetly
            // level -> new walls -> level -> new walls
            // and then the world handler handling it
            // spanning multiple cycles

            // i would think i need it later
            // for handle in ldtk_project_entities {
            // one handle should have every level data already
            // but i cant just ldtk_project_entities[i]

            // println!("{:?}", handle);
            // println!("b4 {:?}", world_handler.loaded_worlds);
            if world_handler.loaded_worlds.contains(level_iid) {
                return Ok(());
            }
            world_handler.loaded_worlds.push(level_iid.clone());
            // println!("4ftr {:?}", world_handler.loaded_worlds);

            // all this to get the level width & height btw
            // and the handle thing aswell
            // let ldtk_project = ldtk_project_assets
            //     .get(handle)
            //     .expect("LdtkProject should be loaded when level is spawned");
            // let level = ldtk_project
            //     .get_raw_level_by_iid(level_iid.get())
            //     .expect("spawned level should exist in project");

            let wall_locations = walls.iter().copied().collect();

            let new_level_walls = LevelWalls { wall_locations };

            *level_walls = new_level_walls.clone();
            println!("{:?}", new_level_walls.wall_locations.len());
            println!("{:?}", level_iid.get());
            // }
        }
    }
    Ok(())
}
