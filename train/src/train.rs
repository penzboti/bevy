use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::Direction;

pub struct TrainPlugin;

impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Carriage {
    pub position: GridCoords,
}

#[derive(Resource)]
pub struct Train {
    pub carriages: Vec<Carriage>,
    pub tracks: Vec<Track>,
}

#[derive(Component)]
pub struct Track {
    pub grid_coords: GridCoords,
    pub direction: Direction,
}
