#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

mod level;
mod player;
mod train;

fn main() {
    App::new()
        // init
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins((level::LevelPlugin, player::PlayerPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, game_tick)
        .add_systems(Update, translate_grid_coords_entities)
        .run();
}

pub const GRID_SIZE: i32 = 16;
const SECONDS_PER_TICK: f32 = 1.;

fn setup(mut commands: Commands) {
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
    commands.insert_resource(train::Train {
        carriages: vec![train::Carriage {
            position: GridCoords::new(0, 0),
        }],
        tracks: vec![],
    });
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum Direction {
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
pub struct GameTickTimer(Timer);

fn game_tick(
    mut player_query: Query<(&mut player::Player, &mut GridCoords)>,
    // mut train: ResMut<Train>,
    time: Res<Time>,
    mut timer: ResMut<GameTickTimer>,
    level_walls: Res<level::LevelWalls>,
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
