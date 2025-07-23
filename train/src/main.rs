#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

mod level;
mod player;
mod train;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins((level::LevelPlugin, player::PlayerPlugin, train::TrainPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, game_tick)
        .add_systems(Update, translate_grid_coords_entities)
        .run();
}

pub const GRID_SIZE: i32 = 16; // pixel size i think
const SECONDS_PER_TICK: f32 = 0.5;
pub const CARRIAGE_NUMBER: isize = 2;

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

// TODO: separate tick system for each plugin (so separate player from here)
fn game_tick(time: Res<Time>, mut timer: ResMut<GameTickTimer>) {
    timer.tick(time.delta());
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
