use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::Direction;
use crate::GameTickTimer;
use crate::level;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_keypress)
            .add_systems(Update, animate_player)
            .add_systems(Update, move_player)
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}

#[derive(Default, Component)]
pub struct Player {
    pub direction: Direction,
    pub previous_direction: Direction, // for track placement
    pub direction_inputs: Vec<Direction>,
}
#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
    #[worldly]
    worldly: Worldly,
}

fn animate_player(mut player_query: Query<(&mut Sprite, &Player), Changed<Player>>) {
    if player_query.is_empty() {
        return;
    }

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

// this is hard because of "koyote time"
// i can hug a wall and follow its direction
// i can pre-press the button and the game just uses that movement
fn handle_keypress(mut player_query: Query<&mut Player>, keys: Res<ButtonInput<KeyCode>>) {
    if let Ok(mut player) = player_query.single_mut() {
        let current_direction = player.direction.clone();

        let pressed_direction: Direction = if keys.just_pressed(KeyCode::KeyW) {
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

        // limiting inputs
        if player.direction_inputs.clone().len() < 2 {
            // dont spam the same input (doesn't matter anyways)
            if let Some(last) = player.direction_inputs.clone().last() {
                if last == &pressed_direction {
                    return;
                }
            }
            player.direction_inputs.push(pressed_direction);
        } else {
            // replacing inputs (if overflow)
            player.direction_inputs.remove(0);
            player.direction_inputs.push(pressed_direction);
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Player, &mut GridCoords)>,
    timer: Res<GameTickTimer>,
    level_walls: Res<level::LevelWalls>,
) {
    if !timer.finished() {
        return;
    }
    if let Ok((mut player, mut grid_coords)) = player_query.single_mut() {
        let current_direction = player.direction.clone();

        // we pop from the back, and we want the first pressed button to come first in line;
        player.direction_inputs.reverse();
        let mut first_action = player
            .direction_inputs
            .pop()
            .unwrap_or(current_direction.clone());
        let second_action = player.direction_inputs.pop();
        if let Some(direction) = second_action.clone() {
            player.direction_inputs.push(direction.clone());
        }

        player.previous_direction = current_direction.clone().get_opposite();

        // not allowing oppsites
        if first_action == current_direction.get_opposite() {
            first_action = current_direction.clone();
        }

        let destination = *grid_coords + first_action.calculate_vector();
        let wall_hug_destination = *grid_coords + current_direction.calculate_vector();

        if !level_walls.in_wall(&destination) {
            // not wall
            *grid_coords = destination;
            player.direction = first_action;
            return;
        }
        // wall, check for hugs
        if !level_walls.in_wall(&wall_hug_destination) {
            player.direction_inputs.insert(0, first_action);
            *grid_coords = wall_hug_destination;
            return;
        }
        // first_action would have been in the wall but the second one isn't
        if let Some(direction) = second_action.clone() {
            // kinda repeating what we did before but luckily we only have to do it this time
            if direction.clone() != current_direction.get_opposite() {
                let destination = *grid_coords + direction.calculate_vector();
                if !level_walls.in_wall(&destination) {
                    *grid_coords = destination;
                    player.direction = direction;
                }
            }
        }
    }
}
