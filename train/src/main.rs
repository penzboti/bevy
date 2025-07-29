#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

mod level;
mod objective;
mod player;
mod train;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins((
            level::LevelPlugin,
            player::PlayerPlugin,
            train::TrainPlugin,
            objective::ObjectivePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, game_tick)
        .add_systems(Update, translate_grid_coords_entities)
        .add_systems(Update, move_camera)
        .run();
}

pub const GRID_SIZE: i32 = 16; // pixel size i think; doesn't affect the world (only entities)
const TICKS_PER_SECOND: i32 = 5;
const SECONDS_PER_TICK: f32 = 1. / TICKS_PER_SECOND as f32;
pub const CARRIAGE_NUMBER: isize = 5;
pub const LEVEL_NUMBER: usize = 10;

fn setup(mut commands: Commands) {
    // easier testing
    #[cfg(target_os = "windows")]
    let scale = 0.5;
    #[cfg(target_os = "linux")]
    let scale = 1.;

    // camera
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale, // bigger the number smaller the world
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(128., 0., 0.),
    ));

    // background color
    commands.insert_resource(ClearColor(Color::srgb(0., 0., 0.5)));
    // init tick system
    commands.insert_resource(GameTickTimer(Timer::from_seconds(
        SECONDS_PER_TICK,
        TimerMode::Repeating,
    )));
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
pub struct GameTickTimer(Timer);

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

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&GridCoords, (With<player::Player>, Changed<GridCoords>)>,
) {
    if player_query.is_empty() || camera_query.is_empty() {
        return;
    }
    let player = player_query.single().unwrap();
    if let Ok(mut camera) = camera_query.single_mut() {
        camera.translation.y = (player.y * GRID_SIZE) as f32;
    }
}
