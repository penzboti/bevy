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
    pub previous_direction: Direction,
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
        } else {
            player.list_next_directions.remove(0);
            player.list_next_directions.push(next_dir);
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Player, &mut GridCoords)>,
    timer: Res<GameTickTimer>,
    level_walls: Res<level::LevelWalls>,
) {
    if timer.finished() {
        if let Ok((mut player, mut grid_coords)) = player_query.single_mut() {
            let current_direction = player.direction.clone();
            println!("{:?}", player.list_next_directions);

            // we pop from the back, and we want the first pressed button to come first in line;
            // TODO: .remove()?
            player.list_next_directions.reverse();
            let mut first_action = player
                .list_next_directions
                .pop()
                .unwrap_or(current_direction.clone());
            let second_action = player.list_next_directions.pop();
            if let Some(direction) = second_action.clone() {
                player.list_next_directions.push(direction.clone());
                // this might not even do anything
                // if first_action == player.direction
                //     || first_action == player.direction.get_opposite()
                // {
                //     if direction != player.direction && direction != player.direction.get_opposite()
                //     {
                //         first_action = direction.clone();
                //     }
                // }
            }

            player.previous_direction = current_direction.clone().get_opposite(); // for track placement

            if first_action == current_direction.get_opposite() {
                first_action = current_direction.clone();
            }

            let destination = *grid_coords + first_action.calculate_vector();
            let wall_hug_destination = *grid_coords + current_direction.calculate_vector();

            if !level_walls.in_wall(&destination) {
                // not wall
                *grid_coords = destination;
                player.direction = first_action;
            } else {
                // wall, check for hugs
                if !level_walls.in_wall(&wall_hug_destination) {
                    player.list_next_directions.insert(0, first_action);
                    // player.list_next_directions.push(first_action);
                    *grid_coords = wall_hug_destination;
                } else {
                    if let Some(direction) = second_action.clone() {
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
        }
    }
}
