use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::Direction;
use crate::GameTickTimer;
use crate::player;

pub struct TrainPlugin;
impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Train {
            carriages: vec![],
            tracks: vec![],
        })
        .register_ldtk_entity::<CarriageBundle>("Carriage")
        .register_ldtk_entity::<TrackBundle>("Track")
        .add_systems(Update, spawn_track)
        .add_systems(Update, init_train)
        .add_systems(Update, move_carriages);
    }
}

// so it would have been cool if i could spawn ldtk entities
// but i just didn't find enough resources to find out if it even exists
// so i'm just spawning regular bevy entities; which are fine.
// although the starting entities are present in ldtk;
// few tracks and a head carriage
#[derive(Default, Resource)]
pub struct Train {
    pub carriages: Vec<GridCoords>,
    pub tracks: Vec<GridCoords>,
}

#[derive(Default, Component)]
pub struct Carriage {}
#[derive(Default, Bundle, LdtkEntity)]
struct CarriageBundle {
    carriage: Carriage,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Debug, Component)]
pub struct Track {
    pub direction_from: Direction,
    pub direction_to: Direction,
}
#[derive(Default, Bundle, LdtkEntity)]
struct TrackBundle {
    track: Track,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

pub fn init_train(
    mut train: ResMut<Train>,
    track_query: Query<&GridCoords, With<Track>>,
    carriage_query: Query<&GridCoords, With<Carriage>>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            for track in track_query {
                train.tracks.push(*track);
            }
            for carriage in carriage_query {
                train.carriages.push(*carriage);
            }
            train.tracks.sort_by(|a, b| a.y.cmp(&b.y));
            println!("{:?}", train.tracks);
            println!("{:?}", train.carriages);
        }
    }
    // TODO: add more carriages
    // NOTE: only the head carriage is spawned in from ldtk
}

fn spawn_track(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut train: ResMut<Train>,
    player_query: Query<(&GridCoords, &player::Player), Changed<GridCoords>>,
) {
    // TODO: const these (or lazy static)
    // let texture = asset_server.load("sprites/Track.png");
    // let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 8, None, None);
    // let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let default_coords = GridCoords::new(0, 0); // borrow checker whining
    if player_query.is_empty() {
        return;
    }
    let (coords, player) = player_query.single().unwrap();

    let coords_count = train
        .tracks
        .iter()
        .filter(|&track| *track == *coords)
        .count();

    if coords_count != 0 {
        // println!("multiple coords");
        return;
    }
    let new_track = Track {
        direction_from: player.previous_direction.clone(),
        direction_to: player.direction.clone(),
    };
    // INFO: https://bevy.org/examples/2d-rendering/sprite-sheet/
    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("sprites/Track.png"),
            TextureAtlas {
                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(16),
                    8,
                    8,
                    None,
                    None,
                )),
                index: 0,
            },
        ),
        Transform {
            ..Default::default()
        },
        coords.clone(),
        new_track,
    ));
    train.tracks.push(coords.clone());
}

// TODO: untested with multiple carriages
fn move_carriages(
    mut train: ResMut<Train>,
    player_query: Query<&player::Player>,
    track_query: Query<&GridCoords, (With<Track>, Without<Carriage>)>,
    mut carriage_query: Query<&mut GridCoords, (With<Carriage>, Without<Track>)>,
    timer: Res<GameTickTimer>,
) {
    if timer.finished() {
        let mut carriages = train.carriages.clone();
        carriages.reverse();
        for i in 1..carriages.len() {
            carriages[i] = carriages[i - 1];
        }
        let current_coords = carriages[0];
        let current_index = train
            .tracks
            .iter()
            .position(|x| x == &current_coords)
            .unwrap();
        let next_coords = train
            .tracks
            .iter()
            .nth(current_index + 1)
            .unwrap_or(&current_coords);

        carriages[0] = *next_coords;
        carriages.reverse();
        for mut coords in carriage_query.iter_mut() {
            // println!("from: {:?}", coords);
            for (index, carriage) in train.carriages.iter().enumerate() {
                if *carriage == coords.clone() {
                    *coords = carriages[index];
                }
            }
            // println!("to: {:?}", coords);
        }
        train.carriages = carriages;
    }
}
