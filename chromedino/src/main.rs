// mostly followed this tutorial: https://github.com/Biped-Potato/flappy_bird/blob/master/src/main.rs

use bevy::prelude::*;
use rand::{Rng,thread_rng};

const PLAIN_HEIGHT: f32 = 0.;

// dino
const JUMP_FORCE: f32 = 500.;
const GRAVITY: f32 = 1000.;
const DINO_HEIGHT: f32 = 60.;
const DINO_WIDTH: f32 = 20.;
// cacti
const OBSTACLE_AMMOUNT: i32 = 3;
const OBSTACLE_WIDTH_MIN: f32 = 20.;
// const OBSTACLE_WIDTH_MAX: f32 = 50.;
const OBSTACLE_HEIGHT_MIN: f32 = 5.;
// const OBSTACLE_HEIGHT_MAX: f32 = 60.;
const OBSTACLE_SCROLL_SPEED: f32 = 100.;
const OBSTACLE_SPACING: f32 = 500.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_canvas)
        .add_systems(Update, update_dino)
        .add_systems(Update, update_obstacles)
        .run();
}

#[derive(Component)]
struct Dino {
    pub velocity: f32,
    pub jumped: bool,
}

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
pub struct GameManager{
    pub window_dimensions: Vec2,
    // gamestate manager (dead)
}

fn setup_canvas(
    mut commands: Commands,
    window_query: Query<&Window>,
) {
    commands.spawn(Camera2d);
    // background color
    commands.insert_resource(ClearColor(Color::srgb(0.,0.,0.)));

    // player
    commands.spawn((
        // a one by one cube of color
        Sprite::from_color(Color::srgb(1.,1.,1.), Vec2::ONE),
        Transform {
            // position
            translation: Vec3::new(00.0, PLAIN_HEIGHT, 0.0),
            // size (multiplying the actual size)
            scale: Vec2::new(DINO_WIDTH,DINO_HEIGHT).extend(1.0),
            ..default()
        },
        Dino { velocity: 0., jumped: false },
    ));

    // obstacles
    let window = window_query.get_single().unwrap();
    commands.insert_resource(GameManager {window_dimensions: Vec2::new(window.width(), window.height())});
    spawn_obstacles(&mut commands, window.width())
}

fn spawn_obstacles(
    mut commands: &mut Commands,
    window_width: f32,
) {
    for i in 0..OBSTACLE_AMMOUNT {
        // let height = generate_rand(rand, OBSTACLE_HEIGHT_MIN, OBSTACLE_HEIGHT_MAX);
        // let width = generate_rand(rand, OBSTACLE_WIDTH_MIN, OBSTACLE_WIDTH_MAX);
        let height = OBSTACLE_HEIGHT_MIN;
        let width = OBSTACLE_WIDTH_MIN;
        let x = window_width / 4. + (OBSTACLE_SPACING * i as f32)-100.;
        let position = Vec3::X * x + Vec3::Y * PLAIN_HEIGHT;
        let size = Vec2::new(width,height);
        spawn_single_obstacle(position, size, &mut commands);
    }
}

fn spawn_single_obstacle(
    position: Vec3,
    size: Vec2,
    mut commands: &mut Commands,
) {
    commands.spawn((
        Sprite::from_color(Color::srgb(1.,0.,0.), Vec2::ONE),
        Transform {
            translation: position,
            scale: size.extend(1.0),
            ..default()
        },
        Obstacle,
    ));
    println!("pos:{:?}\nsize:{:?}\n",position, size);
}

fn _generate_rand(min: f32, max: f32) -> f32 {
    let mut rand = thread_rng();
    return rand.gen_range(min..max)
}

fn update_dino(
    mut dino_query: Query<(&mut Dino,&mut Transform),(With<Dino>,Without<Obstacle>)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    obstacle_query: Query<&Transform, (With<Obstacle>,Without<Dino>)>,
    mut game_manager: ResMut<GameManager>,
) {
    if let Ok((mut dino, mut transform)) = dino_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) && dino.jumped == false{
            dino.jumped = true;
            dino.velocity = JUMP_FORCE;
        }
        dino.velocity -= time.delta_secs() * GRAVITY;
        transform.translation.y += dino.velocity * time.delta_secs();
        if transform.translation.y < 0. {
            dino.velocity = 0.;
            dino.jumped = false;
            transform.translation.y = 0.;
        }
        let mut dead = false;
        if transform.translation.y <= -game_manager.window_dimensions.y / 2. {
            dead = true;
        } else {
            for obs_transform in obstacle_query.iter() {
                //collision check
                if (obs_transform.translation.y - transform.translation.y).abs()
                    < OBSTACLE_HEIGHT_MIN / 2.
                    && (obs_transform.translation.x - transform.translation.x).abs()
                        < OBSTACLE_WIDTH_MIN / 2.
                {
                    dead = true;
                    break;
                }
            }
        }
        if dead { 
            // update game state
            println!("dead");
        }
    }
}

fn update_obstacles(
    // mut commands: &mut Commands,
    mut obstacle_query: Query<&mut Transform,With<Obstacle>>,
    game_manager: Res<GameManager>,
    time: Res<Time>,
) {
    for mut transform in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * OBSTACLE_SCROLL_SPEED;
        // println!("{:?}, x: {:?}",transform.scale, transform.scale.x);
        if transform.translation.x - transform.scale.x / 2. < -game_manager.window_dimensions.x / 2. {
            transform.translation.x += OBSTACLE_AMMOUNT as f32 * OBSTACLE_SPACING;
            transform.translation.y = 0.;
        }
    }
}

fn exit_game(
    game_manager: Res<GameManager>,
    mut exit: EventWriter<AppExit>,
) {
    // if dead { exit.send(AppExit::Success); }
}
