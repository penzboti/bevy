use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::CARRIAGE_NUMBER;
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
// the ldtk entities are automatically load in and i just treat them like bevy entities
// the rest i need i spawn in bevy way
// ldtk: few tracks and a head carriage
#[derive(Default, Resource)]
pub struct Train {
    pub carriages: Vec<GridCoords>,
    pub tracks: Vec<GridCoords>,
}

#[derive(Default, Component)]
pub struct Carriage;
#[derive(Default, Bundle, LdtkEntity)]
struct CarriageBundle {
    carriage: Carriage,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
    #[worldly]
    worldly: Worldly,
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
    #[worldly]
    worldly: Worldly,
}

pub fn init_train(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut train: ResMut<Train>,
    track_query: Query<&GridCoords, With<Track>>,
    carriage_query: Query<&GridCoords, With<Carriage>>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        if let LevelEvent::Spawned(level_iid) = level_event {
            if level_iid != &LevelIid::from(crate::level::START_IID.to_owned()) {
                return;
            }
            for carriage in carriage_query {
                if train.carriages.contains(carriage) {continue;}
                train.carriages.push(*carriage);
            }
            for track in track_query {
                if train.tracks.contains(track) {continue;}
                train.tracks.push(*track);
            }
            train.tracks.sort_by(|a, b| a.y.cmp(&b.y));
            let first_position = train.carriages[0].clone();

            // NOTE: only the head carriage is spawned in from ldtk
            for i in 0..CARRIAGE_NUMBER {
                // we need to space out the carriages at spawn
                let position = first_position + GridCoords::new(0, -1 - i as i32);
                commands.spawn((
                    Sprite::from_atlas_image(
                        asset_server.load("sprites/Carriage.png"),
                        TextureAtlas {
                            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                                UVec2::splat(16),
                                8,
                                8,
                                None,
                                None,
                            )),
                            index: 8,
                        },
                    ),
                    Transform {
                        translation: Vec3::new(0., 0., 999.), // this z index is diabolical i just
                        // didn't want to mess around with it
                        ..Default::default()
                    },
                    position,
                    Carriage {},
                ));
                train.carriages.push(position);
            }
        }
    }
}

fn spawn_track(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut train: ResMut<Train>,
    player_query: Query<(&GridCoords, &player::Player), Changed<GridCoords>>,
) {
    // TODO: const these (or lazy static)
    // although they didn't even work this time (hence the commenting)
    // let texture = asset_server.load("sprites/Track.png");
    // let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 8, 8, None, None);
    // let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if player_query.is_empty() {
        return;
    }
    let (coords, player) = player_query.single().unwrap();

    if train.tracks.clone().contains(coords) { return; }
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

fn move_carriages(
    mut train: ResMut<Train>,
    player_query: Query<&player::Player>,
    track_query: Query<&GridCoords, (With<Track>, Without<Carriage>)>,
    mut carriage_query: Query<&mut GridCoords, (With<Carriage>, Without<Track>)>,
    timer: Res<GameTickTimer>,
) {
    if timer.finished() {
        let mut carriages = train.carriages.clone();
        if carriages.len() == 0 || train.tracks.len() == 0 {return;}

        let len = carriages.clone().len();
        carriages.reverse(); // if lag then don't reverse
        for i in 0..len - 1 {
            carriages[i] = carriages[i + 1];
        }
        carriages.reverse();

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
        let index_table: Vec<usize> = carriage_query
            .iter()
            .clone()
            .map(|coords| train.carriages.iter().position(|c| c == coords).unwrap())
            .collect();
        for (i, mut coords) in carriage_query.iter_mut().enumerate() {
            let index = index_table[i];
            *coords = carriages[index];
        }
        train.carriages = carriages;
    }
}
