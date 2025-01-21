// mostly followed this tutorial: https://github.com/Biped-Potato/flappy_bird/blob/master/src/main.rs

use bevy::prelude::*;
use rand::{Rng,thread_rng};

const PLAIN_HEIGHT: f32 = 0.;

// dino
const JUMP_FORCE: f32 = 500.;
const GRAVITY: f32 = 1300.;
const DINO_HEIGHT: f32 = 60.;
const DINO_WIDTH: f32 = 20.;
// obstacles
const OBSTACLE_AMMOUNT: i32 = 3;
const OBSTACLE_WIDTH_MIN: f32 = 20.;
const OBSTACLE_WIDTH_MAX: f32 = 50.;
const OBSTACLE_HEIGHT_MIN: f32 = 30.;
const OBSTACLE_HEIGHT_MAX: f32 = 74.;
const OBSTACLE_SCROLL_SPEED: f32 = 300.;
const OBSTACLE_SPACING: f32 = 500.;

fn main() {
    App::new()
        // init
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<GameState>()
        .add_systems(Startup, setup_canvas)
        // starter menu
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu_buttons.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), despawn_screen)
        // game
        .add_systems(OnEnter(GameState::Game), (setup_player, setup_obstacles))
        .add_systems(Update, (update_dino, update_obstacles).run_if(in_state(GameState::Game)))
        // death screen
        .add_systems(OnEnter(GameState::Dead), setup_death_screen)
        .add_systems(Update, end_game_button.run_if(in_state(GameState::Dead)))
        .add_systems(OnExit(GameState::Dead), despawn_screen)

        .run();
}

#[derive(Component)]
struct Despawn;

#[derive(Component, PartialEq, States, Debug, Hash, Eq, Clone, Default)]
enum GameState{
    #[default]
    Menu,
    Game,
    Dead,
}

//todo: for all buttons (menus); arrange them
#[derive(Component)]
enum ButtonType {
    Play,
    Exit,
}

#[derive(Component)]
struct Dino {
    pub velocity: f32,
    pub jumped: bool,
}

#[derive(Component)]
struct Obstacle;

//todo: gamespeed controll
#[derive(Resource)]
pub struct GameManager{
    pub window_dimensions: Vec2,
    pub game_speed: f32,
}

fn setup_canvas(
    mut commands: Commands,
    window_query: Query<&Window>,
) {
    // camera
    commands.spawn(Camera2d);
    // background color
    commands.insert_resource(ClearColor(Color::srgb(0.,0.,0.)));
    
    let window = window_query.get_single().unwrap();
    // game_manager
    commands.insert_resource(GameManager {window_dimensions: Vec2::new(window.width(), window.height()),game_speed: 1.});
}

fn setup_menu(
    mut commands: Commands,
) {
    commands.spawn((
        Button, 
        ButtonType::Play,
        Transform::from_translation(Vec3::new(0., 0., 0.)), 
        Despawn
    ))
    .with_child((
        Text::new("Start"), 
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    ));

    // commands.spawn((
    //     Button, 
    //     ButtonType::Exit,
    //     Transform::from_translation(Vec3::new(0., 0., 0.)), 
    //     Despawn
    // ))
    // .with_child((
    //     Text::new("Exit Game"), 
    //     TextColor(Color::srgb(0.9, 0.9, 0.9)),
    // ));
}

fn menu_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    interaction_query: Query< (&Interaction, &ButtonType), (Changed<Interaction>, With<Button>), >,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, button_type) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_type {
                ButtonType::Play => game_state.set(GameState::Game),
                ButtonType::Exit => {exit.send(AppExit::Success);},
            }
        }
    }
}

fn setup_player(
    mut commands: Commands,
    game_manager: Res<GameManager>
) {
    // player
    commands.spawn((
        // a one by one cube of color
        Sprite::from_color(Color::srgb(1.,1.,1.), Vec2::ONE),
        Transform {
            // position
            translation: Vec3::new(00.0, PLAIN_HEIGHT + DINO_HEIGHT / 2., 0.0),
            // size (multiplying the actual size)
            scale: Vec2::new(DINO_WIDTH,DINO_HEIGHT).extend(1.0),
            ..default()
        },
        Dino { velocity: 0., jumped: false },
        Despawn
    ));
    
    // plain line
    commands.spawn((
        Sprite::from_color(Color::srgb(0., 1., 0.),Vec2::ONE),
        Transform {
            translation: Vec3::new(0., PLAIN_HEIGHT, 0.),
            scale: Vec2::new(game_manager.window_dimensions.x, 1.).extend(1.0),
            ..default()
        },
        Despawn)
    );

}

fn setup_obstacles(
    mut commands: Commands,
    game_manager: Res<GameManager>,
) {
    for i in 0..OBSTACLE_AMMOUNT {
        let size = generate_rand();

        let x = game_manager.window_dimensions.x + (OBSTACLE_SPACING * i as f32)-100.;
        let position = Vec3::X * x + Vec3::Y * (PLAIN_HEIGHT + size.y / 2.);

        commands.spawn((
            Sprite::from_color(Color::srgb(1.,0.,0.), Vec2::ONE),
            Transform {
                translation: position,
                scale: size.extend(1.0),
                ..default()
            },
            Obstacle,
            Despawn
        ));
    }
}

fn generate_rand() -> Vec2 {
    let height = Vec2::new(OBSTACLE_HEIGHT_MIN, OBSTACLE_HEIGHT_MAX);
    let width = Vec2::new(OBSTACLE_WIDTH_MIN, OBSTACLE_WIDTH_MAX);
    let mut rand = thread_rng();
    Vec2::new( rand.gen_range(width.x..width.y),
        rand.gen_range(height.x..height.y)
    )
}

fn update_dino(
    mut dino_query: Query<(&mut Dino,&mut Transform),(With<Dino>,Without<Obstacle>)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    obstacle_query: Query<&Transform, (With<Obstacle>,Without<Dino>)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok((mut dino, mut transform)) = dino_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Space) && dino.jumped == false{
            dino.jumped = true;
            dino.velocity = JUMP_FORCE;
        }
        dino.velocity -= time.delta_secs() * GRAVITY;
        transform.translation.y += dino.velocity * time.delta_secs();
        if transform.translation.y < PLAIN_HEIGHT + DINO_HEIGHT / 2. {
            dino.velocity = 0.;
            dino.jumped = false;
            transform.translation.y = PLAIN_HEIGHT + DINO_HEIGHT / 2.;
        }

        for obs_transform in obstacle_query.iter() {
            let (width, height, _) = obs_transform.scale.into();
            //collision check
            if (obs_transform.translation.y - transform.translation.y).abs()
                < height
                && (obs_transform.translation.x - transform.translation.x).abs()
                    < width
            {
                game_state.set(GameState::Dead);
                break;
            }
        }
    }
}

fn update_obstacles(
    mut obstacle_query: Query<&mut Transform,With<Obstacle>>,
    game_manager: Res<GameManager>,
    time: Res<Time>,
) {
    for mut transform in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * OBSTACLE_SCROLL_SPEED * game_manager.game_speed;
        // out of bounds
        if transform.translation.x - transform.scale.x / 2. < -game_manager.window_dimensions.x / 2. - transform.scale.x {
            // "destroy and make a new one"
            // bro just move it back and resize it
            let size = generate_rand();
            transform.translation.x += OBSTACLE_AMMOUNT as f32 * OBSTACLE_SPACING;
            transform.scale = size.extend(1.0);
            transform.translation.y = size.y / 2. + PLAIN_HEIGHT;
        }
    }
}

fn setup_death_screen(
    mut commands: Commands,
) {
    // commands.spawn((Text::new("You died"), TextColor(Color::srgb(1., 0., 0.))));
    commands.spawn((
        Button, 
        Transform::from_translation(Vec3::new(0., 0., 0.)), 
        Despawn
    ))
    .with_child((
        Text::new("Exit"), 
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    ));
}

fn end_game_button(
    mut game_state: ResMut<NextState<GameState>>,
    interaction_query: Query< &Interaction, (Changed<Interaction>, With<Button>), >,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::Menu);
        }
    }
}

fn despawn_screen(
    mut commands: Commands,
    query: Query<Entity, With<Despawn>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}