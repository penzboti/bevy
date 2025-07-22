use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::Direction;
use crate::GameTickTimer;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_keypress)
            .add_systems(Update, animate_player)
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}

#[derive(Default, Component)]
pub struct Player {
    pub direction: Direction,
    pub list_next_directions: Vec<Direction>,
}

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
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
