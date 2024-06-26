use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};
use std::time::Duration;

use bevy::{time::common_conditions::on_timer, window::PrimaryWindow};

use rand::prelude::random;

const ARENA_WIDTH: u32 = 50;
const ARENA_HEIGHT: u32 = 50;

const SNAKE_HEAD_COLOR: Color = Color::rgb(5.0, 5.0, 5.0);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.0, 3.0, 3.0);
const FOOD_COLOR: Color = Color::rgb(3.0, 0.0, 3.0);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component)]
struct Food;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    last_direction: Direction,
    direction: Direction,
}

#[derive(Component)]
struct KeyInput {
    key: KeyCode,
}

#[derive(Event)]
struct GameOverEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (500.0, 500.0).into(),
                title: "Vim_Snake".to_string(),
                ..default()
            }),

            ..default()
        }))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(Time::<Fixed>::from_seconds(1.0))
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, spawn_snake)
        .add_systems(PostUpdate, (position_translation, size_scaling))
        .add_systems(
            Update,
            food_spawner.run_if(on_timer(Duration::from_secs(1))),
        )
        .add_systems(
            Update,
            (
                snake_movement.run_if(on_timer(Duration::from_millis(150))),
                snake_movement_input.before(snake_movement),
                snake_eating.after(snake_movement),
                snake_growth.after(snake_eating),
                game_over.after(snake_movement),
            ),
        )
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,

                    ..default()
                },
                transform: Transform {
                    scale: Vec3::new(10.0, 10.0, 10.0),
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead {
                last_direction: Direction::Up,
                direction: Direction::Up,
            })
            .insert(KeyInput { key: KeyCode::KeyK })
            .insert(SnakeSegment)
            .insert(Position { x: 10, y: 20 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 10, y: 19 }),
    ]);
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum Direction {
    Left,
    Up,
    Down,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

fn snake_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut heads: Query<&mut SnakeHead>,
) {
    if let Some(mut head) = heads.iter_mut().next() {
        // if keyboard_input.pressed(KeyCode::KeyR) {
        //     restart_game();
        // }

        let dir: Direction = if keyboard_input.pressed(KeyCode::KeyL)
            && head.last_direction != Direction::Left
        {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::KeyK) && head.last_direction != Direction::Down {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::KeyJ) && head.last_direction != Direction::Up {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::KeyH) && head.last_direction != Direction::Right {
            Direction::Left
        } else {
            return;
        };

        // let dir: Direction = if keyboard_input.pressed(KeyCode::ArrowLeft)
        //     && head.last_direction != Direction::Right
        // {
        //     Direction::Left
        // } else if keyboard_input.pressed(KeyCode::ArrowUp) && head.last_direction != Direction::Down
        // {
        //     Direction::Up
        // } else if keyboard_input.pressed(KeyCode::ArrowDown) && head.last_direction != Direction::Up
        // {
        //     Direction::Down
        // } else if keyboard_input.pressed(KeyCode::ArrowRight)
        //     && head.last_direction != Direction::Left
        // {
        //     Direction::Right
        // } else {

        //     return;
        // };

        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

fn restart_game() {}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();

        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
        head.last_direction = head.direction;

        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }

        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
    }
}

fn size_scaling(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Size, &mut Transform)>,
) {
    let window = windows.get_single().unwrap();
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Position, &mut Transform)>,
) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_single().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

#[derive(Default, Resource)]
struct LastTailPosition(Option<Position>);

#[derive(Event)]
struct GrowthEvent;

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.read().next().is_some() {
        segments
            .0
            .push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.read().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }

        spawn_snake(commands, segments_res);
    }
}
