#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::collections::HashSet;

fn main() {
    App::new()
        // init
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .register_ldtk_entity::<PlayerBundle>("Player") // can i put this statement anywhere else?
        .register_ldtk_int_cell::<WallBundle>(1) // and this one
        // yes you can its called plugins brotha
        .add_systems(Startup, setup_canvas)
        .add_systems(Startup, setup_level)
        .add_systems(Update, cache_wall_locations)
        .add_systems(
            Update,
            (handle_keypress, game_tick, translate_grid_coords_entities).chain(),
        )
        .add_systems(Update, animate_player)
        .run();
}

const GRID_SIZE: i32 = 16;
const SECONDS_PER_TICK: f32 = 1.;

fn setup_canvas(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(1280.0 / 4.0, 720.0 / 4.0, 0.0),
    ));

    // background color
    commands.insert_resource(ClearColor(Color::srgb(0., 0., 0.)));
    // init tick system
    commands.insert_resource(GameTickTimer(Timer::from_seconds(
        SECONDS_PER_TICK,
        TimerMode::Repeating,
    )));
    // init tracks and train
    commands.insert_resource(Train {
        carriages: vec![Carriage {
            position: Vec2::ZERO,
        }],
        tracks: vec![],
    });
}

#[derive(Default, Component)]
struct Player {
    pub direction: Direction,
    pub list_next_directions: Vec<Direction>,
}

#[derive(Component)]
struct Carriage {
    pub position: Vec2,
}

#[derive(Resource)]
struct Train {
    pub carriages: Vec<Carriage>,
    pub tracks: Vec<Track>,
}

#[derive(Component)]
struct Track {
    pub grid_coords: GridCoords,
    pub direction: Direction,
}

#[derive(Component)]
struct Tile {
    pub grid_coords: GridCoords,
}

#[derive(Default, Clone, PartialEq, Debug)]
enum Direction {
    #[default]
    North,
    South,
    East,
    West,
}

impl Direction {
    fn get_opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
    fn calculate_vector(&self) -> GridCoords {
        match self {
            Direction::North => GridCoords::new(0, 1),
            Direction::West => GridCoords::new(-1, 0),
            Direction::South => GridCoords::new(0, -1),
            Direction::East => GridCoords::new(1, 0),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
struct GameTickTimer(Timer);

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Component)]
struct Wall;

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Default, Resource)]
struct LevelWalls {
    wall_locations: HashSet<GridCoords>,
    level_width: i32,
    level_height: i32,
}

impl LevelWalls {
    fn in_wall(&self, grid_coords: &GridCoords) -> bool {
        grid_coords.x < 0
            || grid_coords.y < 0
            || grid_coords.x >= self.level_width
            || grid_coords.y >= self.level_height
            || self.wall_locations.contains(grid_coords)
    }
}

// #[derive(Default, Bundle, LdtkEntity)]
// struct GoalBundle {
//     #[sprite_sheet_bundle]
//     sprite_sheet_bundle: SpriteSheetBundle,
// }

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load the entire project
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("train.ldtk").into(),
        ..Default::default()
    });
    // select initial level
    commands.insert_resource(LevelSelection::index(0));
    // setup entities
    // commands.regiset_ldtk_entity::<PlayerBundle>("Player");
    commands.init_resource::<LevelWalls>();
}

// this is hard because of "koyote time"
// i can hug a wall and follow its direction
// i can pre-press the button and the game just uses that movement
fn handle_keypress(mut player_query: Query<&mut Player>, keys: Res<ButtonInput<KeyCode>>) {
    if let Ok(mut player) = player_query.single_mut() {
        let dir = player.direction.clone();
        let next_dir: Direction = if keys.just_pressed(KeyCode::KeyW) {
            Direction::North
        } else if keys.just_pressed(KeyCode::KeyA) {
            Direction::West
        } else if keys.just_pressed(KeyCode::KeyS) {
            Direction::South
        } else if keys.just_pressed(KeyCode::KeyD) {
            Direction::East
        } else {
            return;
        };

        if player.list_next_directions.clone().len() < 2 {
            if let Some(last) = player.list_next_directions.clone().last() {
                if last == &next_dir {
                    return;
                }
            }
            player.list_next_directions.push(next_dir);
        }
    }
}

fn game_tick(
    mut player_query: Query<(&mut Player, &mut GridCoords)>,
    // mut train: ResMut<Train>,
    time: Res<Time>,
    mut timer: ResMut<GameTickTimer>,
    level_walls: Res<LevelWalls>,
) {
    timer.tick(time.delta());

    if timer.finished() {
        if let Ok((mut player, mut grid_coords)) = player_query.single_mut() {
            // can't pop_front, so reversing then reverse back
            player.list_next_directions.reverse();
            let mut attempted_direction = player
                .list_next_directions
                .pop()
                .unwrap_or(player.direction.clone())
                .clone();
            player.list_next_directions.reverse();

            let current_direction = player.direction.clone();

            if attempted_direction == current_direction.get_opposite() {
                attempted_direction = current_direction.clone();
            }

            let destination = *grid_coords + attempted_direction.calculate_vector();

            if !level_walls.in_wall(&destination) {
                // not wall
                *grid_coords = destination;
                player.direction = attempted_direction;
            } else {
                // wall, check for hugs
                let wall_hug_destination = *grid_coords + current_direction.calculate_vector();
                if !level_walls.in_wall(&wall_hug_destination) {
                    player.list_next_directions.insert(0, attempted_direction);
                    *grid_coords = wall_hug_destination;
                }
            }
        }
    }
}

fn translate_grid_coords_entities(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in grid_coords_entities.iter_mut() {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}

fn cache_wall_locations(
    mut level_walls: ResMut<LevelWalls>,
    mut level_events: EventReader<LevelEvent>,
    walls: Query<&GridCoords, With<Wall>>,
    ldtk_project_entities: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) -> Result {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_project_entities.single()?)
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

            *level_walls = new_level_walls;
        }
    }
    Ok(())
}

fn animate_player(mut player_query: Query<(&mut Sprite, &Player)>, timer: Res<GameTickTimer>) {
    if timer.finished() {
        let (mut sprite, player) = player_query.single_mut().unwrap();
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = match player.direction {
                Direction::West => 8,
                Direction::South => 9,
                Direction::North => 10,
                Direction::East => 11,
            };
        }
    }
}
