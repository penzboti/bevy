use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<TravellerBundle>("Traveller")
            .register_ldtk_entity::<StationBundle>("Station");
    }
}

#[derive(Default, Bundle, LdtkEntity)]
struct TravellerBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Bundle, LdtkEntity)]
struct StationBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}
