use rand::{thread_rng, Rng};

use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_prototype_lyon::prelude::*;

const WINDOW_W: f32 = 420.0;
const WINDOW_H: f32 = 420.0;

struct GameWorld {
  grid_size: f32,
}

struct Snake {
  score: u32,
}
struct SnakeBody {
  index: u32,
}
struct SnakeMoveTimer(Timer);
struct Velocity(Vec2);

struct Moved(bool);
struct MoveQueue(Vec<Vec2>);

struct Food;

struct GrowEvent(Transform);
struct GameOverEvent;

struct ScoreText;

fn main() {
  App::build()
    .add_event::<GrowEvent>()
    .add_event::<GameOverEvent>()
    .insert_resource(Moved(true))
    .insert_resource(MoveQueue(vec![Vec2::splat(0.0)]))
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(GameWorld { grid_size: 20.0 })
    .insert_resource(SnakeMoveTimer(Timer::from_seconds(0.1, true)))
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_startup_system(ui_setup.system())
    .add_startup_system(setup.system())
    .add_startup_system(spawn_snake.system())
    .add_system(spawn_food.system())
    .add_system(keyboard_input.system().label("Movement"))
    .add_system(update.system().after("Movement"))
    .add_system(eat_food.system())
    .add_system(grow_snake.system())
    .add_system(reset_game.system())
    .run();
}

fn setup(mut windows: ResMut<Windows>, mut commands: Commands) {
  let window = windows.get_primary_mut().unwrap();
  window.set_resolution(WINDOW_W, 420.0);
  window.set_resizable(false);

  commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_snake(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let ball_handle = asset_server.load("snowball.png");
  commands
    .spawn()
    .insert_bundle(SpriteBundle {
      material: materials.add(ball_handle.into()),
      transform: Transform {
        translation: Vec3::splat(0.0),
        scale: Vec3::new(1.5, 1.5, 0.0),
        ..Default::default()
      },
      ..Default::default()
    })
    .insert(Snake { score: 0 })
    .insert(Velocity(Vec2::ZERO));
}

fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn_bundle(UiCameraBundle::default());
  commands
    .spawn_bundle(TextBundle {
      style: Style {
        align_self: AlignSelf::Center,
        position_type: PositionType::Absolute,
        position: Rect {
          left: Val::Px(0.0),
          top: Val::Px(0.0),
          ..Default::default()
        },
        ..Default::default()
      },
      text: Text::with_section(
        "Score: 0",
        TextStyle {
          font: asset_server.load("fonts/ARIAL.ttf"),
          font_size: 30.0,
          color: Color::WHITE,
        },
        Default::default(),
      ),
      ..Default::default()
    })
    .insert(ScoreText);
}

fn update(
  world: Res<GameWorld>,
  time: Res<Time>,
  mut timer: ResMut<SnakeMoveTimer>,
  mut query: Query<(&mut Transform, &Velocity), (With<Snake>, Without<SnakeBody>)>,
  mut snake_body: Query<(&SnakeBody, &mut Transform), With<SnakeBody>>,
  mut game_over: EventWriter<GameOverEvent>,
  mut moved: ResMut<Moved>,
) {
  if timer.0.tick(time.delta()).just_finished() {
    if let Ok((mut transform, velocity)) = query.single_mut() {
      let old_transform = transform.clone();

      transform.translation.y += velocity.0.y * world.grid_size;
      transform.translation.x += velocity.0.x * world.grid_size;

      moved.0 = true;

      if transform.translation.x > WINDOW_W / 2.
        || transform.translation.x < -WINDOW_W / 2.
        || transform.translation.y < -WINDOW_H / 2.
        || transform.translation.y > WINDOW_H / 2.
      {
        println!("dead");
        game_over.send(GameOverEvent);
      }

      let mut snake_bodies: Vec<(u32, Vec3)> = Vec::new();
      for (body, body_transform) in snake_body.iter_mut() {
        if transform.translation == body_transform.translation {
          println!("dead");
          game_over.send(GameOverEvent);
        }
        snake_bodies.push((body.index, body_transform.translation));
      }

      snake_bodies.sort_by_key(|k| k.0);

      for (body, mut transform) in snake_body.iter_mut() {
        let index = body.index as usize;
        if index == 0 {
          transform.translation = old_transform.translation
        } else {
          transform.translation = snake_bodies[index - 1].1
        }
      }
    }
  }
}

fn keyboard_input(
  keyboard_input: Res<Input<KeyCode>>,
  mut query: Query<&mut Velocity, With<Snake>>,
  mut moved: ResMut<Moved>,
  mut move_queue: ResMut<MoveQueue>,
) {
  if let Ok(mut velocity) = query.single_mut() {
    let mut new_velocity = velocity.0;
    if keyboard_input.pressed(KeyCode::W) {
      if velocity.0 != Vec2::new(0.0, -1.0) {
        new_velocity = Vec2::new(0.0, 1.0);
      }
    } else if keyboard_input.pressed(KeyCode::S) {
      if velocity.0 != Vec2::new(0.0, 1.0) {
        new_velocity = Vec2::new(0.0, -1.0);
      }
    } else if keyboard_input.pressed(KeyCode::A) {
      if velocity.0 != Vec2::new(1.0, 0.0) {
        new_velocity = Vec2::new(-1.0, 0.0);
      }
    } else if keyboard_input.pressed(KeyCode::D) {
      if velocity.0 != Vec2::new(-1.0, 0.0) {
        new_velocity = Vec2::new(1.0, 0.0);
      }
    }
    if moved.0 == true {
      if move_queue.0[0] != Vec2::splat(0.0) {
        velocity.0 = move_queue.0[0];
        move_queue.0[0] = Vec2::splat(0.0);
      }
      velocity.0 = new_velocity;
      moved.0 = false;
    } else if new_velocity != velocity.0 {
      move_queue.0[0] = new_velocity;
    }
  }
}

fn spawn_food(
  mut commands: Commands,
  world: Res<GameWorld>,
  food: Query<&Food>,
  snake: Query<&Transform, Or<(With<Snake>, With<SnakeBody>)>>,
) {
  if let Err(_) = food.single() {
    let snake_pos: Vec<Vec3> = snake.iter().map(|m| m.translation).collect();
    let mut search_new = true;
    let mut rng = thread_rng();
    let shape = shapes::RegularPolygon {
      sides: 4,
      feature: shapes::RegularPolygonFeature::SideLength(world.grid_size),
      ..shapes::RegularPolygon::default()
    };

    while search_new == true {
      let x =
        rng.gen_range(-WINDOW_W / 2.0 / world.grid_size + 1.0..WINDOW_W / 2.0 / world.grid_size);
      let y =
        rng.gen_range(-WINDOW_H / 2.0 / world.grid_size + 1.0..WINDOW_H / 2.0 / world.grid_size);
      let x = x.floor() * world.grid_size;
      let y = y.floor() * world.grid_size;

      let xy_compare = Vec3::new(x, y, 0.0);
      if snake_pos.contains(&xy_compare) {
        search_new = true
      } else {
        search_new = false;
        commands
          .spawn()
          .insert_bundle(GeometryBuilder::build_as(
            &shape,
            ShapeColors::outlined(Color::RED, Color::RED),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_xyz(x, y, 0.0),
          ))
          .insert(Food);
      }
    }
  }
}

fn eat_food(
  mut snake: Query<(&mut Snake, &Transform), With<Snake>>,
  food: Query<(Entity, &Transform), With<Food>>,
  mut score_text: Query<&mut Text, With<ScoreText>>,
  mut commands: Commands,
  mut ev_grow: EventWriter<GrowEvent>,
) {
  if let Ok((mut snake, snake_transform)) = snake.single_mut() {
    if let Ok((entity, food_transform)) = food.single() {
      if snake_transform.translation.xy() == food_transform.translation.xy() {
        snake.score += 1;
        ev_grow.send(GrowEvent(snake_transform.clone()));
        commands.entity(entity).despawn();
        if let Ok(mut text) = score_text.single_mut() {
          text.sections[0].value = format!("Score: {}", snake.score);
        }
      }
    }
  }
}

fn grow_snake(
  mut commands: Commands,
  snake_bodies: Query<&SnakeBody>,
  mut ev_grow: EventReader<GrowEvent>,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for ev in ev_grow.iter() {
    let body_parts = snake_bodies.iter().len();
    let mut ball_transform = ev.0;
    ball_transform.scale = Vec3::new(1.5, 1.5, 0.0);
    let ball_handle = asset_server.load("snowball.png");
    commands
      .spawn()
      .insert_bundle(SpriteBundle {
        sprite: Sprite {
          size: Vec2::splat(2.),
          ..Default::default()
        },
        material: materials.add(ball_handle.into()),
        transform: ball_transform,
        ..Default::default()
      })
      .insert(SnakeBody {
        index: body_parts as u32,
      })
      .insert(Velocity(Vec2::ZERO));
  }
}

fn reset_game(
  mut commands: Commands,
  mut reader: EventReader<GameOverEvent>,
  snake: Query<Entity, Or<(With<Snake>, With<SnakeBody>)>>,
  asset_server: Res<AssetServer>,
  materials: ResMut<Assets<ColorMaterial>>,
  mut score_text: Query<&mut Text, With<ScoreText>>,
) {
  if reader.iter().next().is_some() {
    for entity in snake.iter() {
      commands.entity(entity).despawn();
    }
    spawn_snake(commands, asset_server, materials);
    if let Ok(mut text) = score_text.single_mut() {
      text.sections[0].value = format!("Score: 0");
    }
  }
}
