// mostly followed this tutorial: https://github.com/Biped-Potato/flappy_bird/blob/master/src/main.rs

use bevy::prelude::*;
use rand::{Rng,thread_rng};
use bevy::math::bounding::{Aabb2d, IntersectsVolume};

// game
const PLAIN_HEIGHT: f32 = 0.;
const SECONDS_UNTIL_FULL_SPEED: f32 = 60.;
const OBSTACLE_SCROLL_SPEED_MIN: f32 = 300.;
const OBSTACLE_SCROLL_SPEED_MAX: f32 = 500.;

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
const OBSTACLE_SPACING_MAX: f32 = 400.;
const OBSTACLE_SPACING: f32 = 500.;

fn main() {
    //todo: rotate camera in the endgame (at around 100sec prob); rotate it more randomly with time
    //todo: add a score system (save file)
    //todo: add assets (not sure if it would work with random width & height but i guess we'll see)

    App::new()
        // init
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<GameState>()
        .add_systems(Startup, setup_canvas)
        
        // windows inupt delay fix
        // the goat: https://spelcodes.nl/how-to-fix-bevy-input-delay-a-complete-troubleshooting-guide/
        .add_plugins(bevy_framepace::FramepacePlugin) 

        // update every state
        .add_systems(Update, hover_buttons)

        // starter menu
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu_buttons.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), despawn_screen)
        // game
        .add_systems(OnEnter(GameState::Game), (setup_player, setup_obstacles))
        .add_systems(Update, ((update_game_speed, update_obstacles, update_dino).chain()).run_if(in_state(GameState::Game)))
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

#[derive(Resource, Deref, DerefMut)]
struct GameSpeedTimer(Timer);

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
    commands.spawn((Camera2d, Transform {..default()}));
    // background color
    commands.insert_resource(ClearColor(Color::srgb(0.,0.,0.)));
    
    let window = window_query.get_single().unwrap();
    // game_manager
    commands.insert_resource(GameManager {window_dimensions: Vec2::new(window.width(), window.height()),game_speed: 1.});

    // game speed timer
    commands.insert_resource(GameSpeedTimer(Timer::from_seconds(SECONDS_UNTIL_FULL_SPEED, TimerMode::Once)));
}

fn hover_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &Children),
        (Changed<Interaction>, With<Button>)
    >,
    mut text_color_query: Query<&mut TextColor>,
) {
    for (interaction, mut background_color, mut border_color, children) in &mut interaction_query {
        let mut text_color = text_color_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::WHITE).into();
                border_color.0 = Color::BLACK;
                **text_color = *TextColor(Color::BLACK);
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::BLACK).into();
                border_color.0 = Color::WHITE;
                **text_color = *TextColor(Color::WHITE);
            }
            _ => {}
        }
    }
}

fn setup_menu(
    mut commands: Commands,
) {
    commands
    // center ui
    .spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }, Despawn))
    .with_children(|parent| {
        // play button
        parent.spawn((
            Button, 
            Node {
                width: Val::Px(100.0),
                height: Val::Px(50.0),
                // rectangle border
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                // space between this and other ui elements (same level)
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(10.0)),
            ButtonType::Play,
        ))
        .with_child((
            Text::new("Start"), 
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        // exit button
        parent.spawn((
            Button, 
            Node {
                width: Val::Auto,
                height: Val::Px(50.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BorderColor(Color::WHITE),
            BorderRadius::all(Val::Px(10.0)),
            ButtonType::Exit,
        ))
        .with_child((
            Text::new("Exit Game"), 
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
}

fn menu_buttons(
    mut game_state: ResMut<NextState<GameState>>,
    interaction_query: Query< (&Interaction, &ButtonType), (Changed<Interaction>, With<Button>), >,
    keys: Res<ButtonInput<KeyCode>>,
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
    if keys.just_pressed(KeyCode::Space){
        game_state.set(GameState::Game);
    }
    if keys.just_pressed(KeyCode::Escape){
        exit.send(AppExit::Success);
    }
}

fn update_game_speed(
    mut game_manager: ResMut<GameManager>,
    time: Res<Time>,
    mut timer: ResMut<GameSpeedTimer>,
) {
    timer.tick(time.delta());
    // https://stackoverflow.com/questions/13462001/ease-in-and-ease-out-animation-formula
    game_manager.game_speed = (timer.elapsed_secs() / SECONDS_UNTIL_FULL_SPEED).powi(2);
}

fn setup_player(
    mut commands: Commands,
    game_manager: Res<GameManager>,
    mut game_speed_timer: ResMut<GameSpeedTimer>,
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

    game_speed_timer.0.reset();
}

fn setup_obstacles(
    mut commands: Commands,
    game_manager: Res<GameManager>,
) {
    for i in 0..OBSTACLE_AMMOUNT {
        let (size, spacing) = generate_rand(1.0);

        let x = game_manager.window_dimensions.x + (OBSTACLE_SPACING * i as f32) + spacing -100.;
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

fn generate_rand(spacing_percent: f32) -> (Vec2,f32) {
    let height = Vec2::new(OBSTACLE_HEIGHT_MIN, OBSTACLE_HEIGHT_MAX);
    let width = Vec2::new(OBSTACLE_WIDTH_MIN, OBSTACLE_WIDTH_MAX);
    let mut rand = thread_rng();
    (Vec2::new(
        rand.gen_range(width.x..width.y).floor(),
        rand.gen_range(height.x..height.y).floor()
    ), (rand.gen_range(0f32..OBSTACLE_SPACING_MAX)*spacing_percent).floor() - OBSTACLE_SPACING_MAX/2.
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
        if keys.just_pressed(KeyCode::KeyQ){
            game_state.set(GameState::Dead);
        }
        dino.velocity -= time.delta_secs() * GRAVITY;
        transform.translation.y += dino.velocity * time.delta_secs();
        if transform.translation.y < PLAIN_HEIGHT + DINO_HEIGHT / 2. {
            dino.velocity = 0.;
            dino.jumped = false;
            transform.translation.y = PLAIN_HEIGHT + DINO_HEIGHT / 2.;
        }

        for obs_transform in obstacle_query.iter() {
            if Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ).intersects(&Aabb2d::new(
                obs_transform.translation.truncate(),
                obs_transform.scale.truncate() / 2.,
            ))
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
    timer: Res<GameSpeedTimer>,
    time: Res<Time>,
) {
    for mut transform in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * ((OBSTACLE_SCROLL_SPEED_MAX - OBSTACLE_SCROLL_SPEED_MIN) * game_manager.game_speed + OBSTACLE_SCROLL_SPEED_MIN);
        // out of bounds
        if transform.translation.x - transform.scale.x / 2. < -game_manager.window_dimensions.x / 2. - transform.scale.x {
            // "destroy and make a new one"
            // bro just move it back and resize it
            let (size, spacing) = generate_rand(1.0 - timer.elapsed_secs() / SECONDS_UNTIL_FULL_SPEED);
            transform.translation.x += OBSTACLE_AMMOUNT as f32 * OBSTACLE_SPACING + spacing;
            transform.scale = size.extend(1.0);
            transform.translation.y = size.y / 2. + PLAIN_HEIGHT;
        }
    }
}

fn rotate_camera(
    mut query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    keys.get_pressed().for_each(|key| {
        if key == &KeyCode::KeyR {
            for mut transform in query.iter_mut() {
                transform.rotate(Quat::from_rotation_z(time.delta_secs()));
            }
        }
    });
}

fn setup_death_screen(
    mut commands: Commands,
) {
    commands
    .spawn((Node {
        width: Val::Percent(100.0),
        height: Val::Percent(85.0),
        flex_direction: FlexDirection::Column,
        // flex-dir flips these two values
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexEnd,
        ..default()
    }, Despawn))
    .with_children(|parent| {
        parent.spawn((
            Text::new("You died"), 
            TextColor(Color::srgb(1., 0., 0.)), 
        ));
        parent.spawn(Node {
            // width: Val::Percent(100.0),
            // height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Button, 
                Node {
                    // width: Val::Px(100.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(20.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::BLACK),
                BorderRadius::all(Val::Px(10.0)),
                ButtonType::Play,
            ))
            .with_child((
                Text::new("Play again"), 
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
            parent.spawn((
                Button, 
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::BLACK),
                BorderRadius::all(Val::Px(10.0)),
                ButtonType::Exit
            ))
            .with_child((
                Text::new("Menu"), 
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
    });
}

fn end_game_button(
    mut game_state: ResMut<NextState<GameState>>,
    interaction_query: Query< (&Interaction, &ButtonType), (Changed<Interaction>, With<Button>), >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (interaction, button_type) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button_type {
                ButtonType::Play => game_state.set(GameState::Game),
                ButtonType::Exit => game_state.set(GameState::Menu),
            }
        }
    }
    if keys.just_pressed(KeyCode::Space){
        game_state.set(GameState::Game);
    }
    if keys.just_pressed(KeyCode::Escape){
        game_state.set(GameState::Menu);
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