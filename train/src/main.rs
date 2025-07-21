use bevy::prelude::*;

fn main() {
    App::new()
        // init
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_canvas)
        .add_systems(Startup, (setup_grid, setup_player).chain())
        .add_systems(Update, (handle_keypress, game_tick).chain())
        .run();
}

const GRID_SIZE: f32 = 40.;
const SECONDS_PER_TICK: f32 = 1.;

fn setup_canvas(
    mut commands: Commands,
) {
    // camera
    commands.spawn(Camera2d);
    // background color
    commands.insert_resource(ClearColor(Color::srgb(0.,0.,0.)));
    // init tick system
    commands.insert_resource(GameTickTimer(Timer::from_seconds(SECONDS_PER_TICK, TimerMode::Repeating)));
    // init tracks and train
    commands.insert_resource(Train {
        carriages: vec![Carriage { position: Vec2::ZERO }],
        tracks: vec![]
    });
}

#[derive(Component)]
struct Player {
    pub position: Vec2,
    pub direction: Direction,
    pub list_next_directions: Vec<Direction>
}

#[derive(Component)]
struct Carriage {
    pub position: Vec2,
}

#[derive(Resource)]
struct Train {
    pub carriages: Vec<Carriage>,
    pub tracks: Vec<Track>,
}

#[derive(Component)]
struct Track {
    pub position: Vec2,
    pub direction: Direction
}

#[derive(Component)]
struct Tile {
    pub position: Vec2,
}

#[derive(Clone, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West
}

#[derive(Resource, Deref, DerefMut)]
struct GameTickTimer(Timer);

fn setup_player(
    mut commands: Commands,
) {
    // player
    commands.spawn((
        // a one by one cube of color
        Sprite::from_color(Color::srgb(1.,1.,1.), Vec2::ONE),
        Transform {
            // position
            translation: Vec3::ZERO,
            // size (multiplying the actual size)
            scale: Vec2::new(GRID_SIZE,GRID_SIZE).extend(1.0),
            // scale: Vec3::ONE,
            ..default()
        },
        Player {
            position: Vec2::ZERO,
            direction: Direction::North,
            list_next_directions: vec![],
        },
    ));    
}

fn setup_grid (
    mut commands: Commands,
) {
    for i in 0..=10 {
        for j in 0..=10 {
            let n1 = i as f32;
            let n2 = j as f32;
            let color = if (i+j) % 2 == 0 { Color::srgb(0.5, 0., 0.) } else { Color::srgb(0., 0., 0.5) };
            commands.spawn((
                Sprite::from_color(color, Vec2::ONE),
                Transform {
                    translation: Vec3::new(GRID_SIZE*n1, GRID_SIZE*n2, 0.),
                    scale: Vec2::new(GRID_SIZE,GRID_SIZE).extend(1.),
                    ..default()
                },
                Tile {
                    position: Vec2 { x: n1, y: n2 }}
            ));
        }
    }
}

// this is hard because of "koyote time"
// i can hug a wall and follow its direction
// i can pre-press the button and the game just uses that movement
fn handle_keypress(
    mut player_query: Query<&mut Player>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut player) = player_query.single_mut() {
        let dir = player.direction.clone();
        if keys.just_pressed(KeyCode::KeyW){
            if dir == Direction::North {player.list_next_directions = vec![];}
            if dir != Direction::South { player.list_next_directions.push(Direction::North); }
        }
        if keys.just_pressed(KeyCode::KeyA){
            if dir == Direction::West {player.list_next_directions = vec![];}
            if dir != Direction::East { player.list_next_directions.push(Direction::West); }
        }
        if keys.just_pressed(KeyCode::KeyS){
            if dir == Direction::South {player.list_next_directions = vec![];}
            if dir != Direction::North { player.list_next_directions.push(Direction::South); }
        }
        if keys.just_pressed(KeyCode::KeyD){
            if dir == Direction::East {player.list_next_directions = vec![];}
            if dir != Direction::West { player.list_next_directions.push(Direction::East); }
        }
    }
}

fn game_tick(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut train: ResMut<Train>,
    time: Res<Time>,
    mut timer: ResMut<GameTickTimer>,
) {
       timer.tick(time.delta());

    if timer.finished() {
        if let Ok((mut player, mut transform)) = player_query.single_mut() {
            player.direction = player.list_next_directions.pop().unwrap_or(&player.direction.clone()).clone();
            if (true) {
                // if a wall is present and the last action was to ride that, then don't clear
                // for that i need the map (loaded and queried)
                player.list_next_directions = vec![];
            }
            let dir = player.direction.clone();
            train.tracks.push(Track{
                position: player.position.clone(),
                direction: dir.clone(),
            });
            player.position += Vec2::from(match dir {
                Direction::North => (0.,1.),
                Direction::West => (-1.,0.),
                Direction::South => (0.,-1.),
                Direction::East => (1.,0.)
            });
            transform.translation = Vec3::new(player.position.x as f32 * GRID_SIZE,player.position.y as f32 * GRID_SIZE, 0.);
        }
    }
}
