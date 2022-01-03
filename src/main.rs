use rand::{thread_rng, Rng};

use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_prototype_lyon::prelude::*;

const WINDOW_W: f32 = 420.0;
const WINDOW_H: f32 = 420.0;

struct GameWorld {
  grid_size: f32,
}

struct LastKey(KeyCode);

struct Snake {
  score: u32
}
struct SnakeBody{
  index: u32
}
struct SnakeMoveTimer(Timer);
struct Velocity(Vec2);

struct Food;

struct GrowEvent;

struct ScoreText;

fn main() {
  App::build()
    .add_event::<GrowEvent>()
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .insert_resource(GameWorld { grid_size: 20.0 })
    .insert_resource(SnakeMoveTimer(Timer::from_seconds(0.1, true)))
    .insert_resource(LastKey(KeyCode::Z))
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_startup_system(ui_setup.system())
    .add_startup_system(setup.system())
    .add_startup_system(spawn_food.system())
    .add_system(update.system())
    .add_system(keyboard_input.system())
    .add_system(eat_food.system())
    .add_system(grow_snake.system())
    .run();
}

fn setup(mut windows: ResMut<Windows>, world: Res<GameWorld>, mut commands: Commands) {
  let window = windows.get_primary_mut().unwrap();
  window.set_resolution(WINDOW_W, 420.0);
  window.set_resizable(false);

  commands.spawn_bundle(OrthographicCameraBundle::new_2d());

  let shape = shapes::RegularPolygon {
    sides: 4,
    feature: shapes::RegularPolygonFeature::SideLength(world.grid_size),
    ..shapes::RegularPolygon::default()
  };
  commands
    .spawn()
    .insert_bundle(GeometryBuilder::build_as(
      &shape,
      ShapeColors::outlined(Color::WHITE, Color::WHITE),
      DrawMode::Fill(FillOptions::default()),
      Transform::from_xyz(0.0, 0.0, 0.0),
    ))
    .insert(Snake {score: 0})
    .insert(Velocity(Vec2::ZERO));
}

fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn_bundle(UiCameraBundle::default());
  commands
    .spawn_bundle(TextBundle {
      style: Style {
        align_self: AlignSelf::Center,
        position_type: PositionType::Absolute,
        position: Rect { left: Val::Px(0.0), top: Val::Px(0.0), ..Default::default() },
        ..Default::default()
      },
      text: Text::with_section(
        "Score: 0",
        TextStyle {
          font: asset_server.load("fonts/ARIAL.ttf"),
          font_size: 30.0,
          color: Color::WHITE,
        },
        Default::default()
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
) {
  if timer.0.tick(time.delta()).just_finished() {
    if let Ok((mut transform, velocity)) = query.single_mut() {
      let old_transform = transform.clone();

      transform.translation.x += velocity.0.x * world.grid_size;
      transform.translation.y += velocity.0.y * world.grid_size;

      let mut snake_bodies: Vec<(u32, Vec3)> = Vec::new();
      for (body, transform) in snake_body.iter_mut() {
        snake_bodies.push((body.index, transform.translation))
      }
  
      snake_bodies.sort_by_key(|k| k.0);
  
      for (body, mut transform) in snake_body.iter_mut() {
        let index = body.index as usize;
        if index == 0 {
          transform.translation = old_transform.translation
        } else {
          transform.translation = snake_bodies[index-1].1
        }
      }
    }
  }
}

fn keyboard_input(
  keyboard_input: Res<Input<KeyCode>>,
  mut query: Query<&mut Velocity, With<Snake>>,
  mut last_key: ResMut<LastKey>
) {
  if let Ok(mut velocity) = query.single_mut() {
    if keyboard_input.pressed(KeyCode::W) {
      if last_key.0 != KeyCode::S {
        velocity.0 = Vec2::new(0.0, 1.0);
        last_key.0 = KeyCode::W;
      }
    }
    if keyboard_input.pressed(KeyCode::S) {
      if last_key.0 != KeyCode::W {
        velocity.0 = Vec2::new(0.0, -1.0);
        last_key.0 = KeyCode::S;
      }
    }
    if keyboard_input.pressed(KeyCode::A) {
      if last_key.0 != KeyCode::D {
        velocity.0 = Vec2::new(-1.0, 0.0);
        last_key.0 = KeyCode::A;
      }
    }
    if keyboard_input.pressed(KeyCode::D) {
      if last_key.0 != KeyCode::A {
        velocity.0 = Vec2::new(1.0, 0.0);
        last_key.0 = KeyCode::D;
      }
    }
  }
}

fn spawn_food(mut commands: Commands, world: Res<GameWorld>) {
  
  let mut rng = thread_rng();

  let shape = shapes::RegularPolygon {
    sides: 4,
    feature: shapes::RegularPolygonFeature::SideLength(world.grid_size),
    ..shapes::RegularPolygon::default()
  };

  let x = rng.gen_range(-WINDOW_W/2.0/world.grid_size+1.0..WINDOW_W/2.0/world.grid_size);
  let y = rng.gen_range(-WINDOW_H/2.0/world.grid_size+1.0..WINDOW_H/2.0/world.grid_size);
  let x = x.floor()*world.grid_size;
  let y = y.floor()*world.grid_size;

  commands
    .spawn()
    .insert_bundle(GeometryBuilder::build_as(
      &shape,
      ShapeColors::outlined(Color::RED, Color::RED),
      DrawMode::Fill(FillOptions::default()),
      Transform::from_xyz(
        x,
        y,
        0.0),
    ))
    .insert(Food);
}

fn eat_food(
  mut snake: Query<(&mut Snake, &Transform), With<Snake>>,
  food: Query<(Entity, &Transform), With<Food>>,
  mut score_text: Query<&mut Text, With<ScoreText>>,
  world: Res<GameWorld>,
  mut commands: Commands,
  mut ev_grow: EventWriter<GrowEvent>
) {
  if let Ok((mut snake, snake_transform)) = snake.single_mut() {
    if let Ok((entity, food_transform)) = food.single() {
      if snake_transform.translation.xy() == food_transform.translation.xy() {
        snake.score += 1;
        ev_grow.send(GrowEvent);
        commands.entity(entity).despawn();
        spawn_food(commands, world);
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
  world: Res<GameWorld>,
  snake: Query<(&Transform, &Velocity), With<Snake>>,
  mut ev_grow: EventReader<GrowEvent>
) {

  for _ in ev_grow.iter() {
    let body_parts = snake_bodies.iter().len();
  
    let new_pos_x = snake.single().unwrap().0.translation.x - snake.single().unwrap().1.0.x;
    let new_pos_y = snake.single().unwrap().0.translation.y - snake.single().unwrap().1.0.y;
  
    let shape = shapes::RegularPolygon {
      sides: 4,
      feature: shapes::RegularPolygonFeature::SideLength(world.grid_size),
      ..shapes::RegularPolygon::default()
    };
    commands
      .spawn()
      .insert_bundle(GeometryBuilder::build_as(
        &shape,
        ShapeColors::outlined(Color::WHITE, Color::WHITE),
        DrawMode::Fill(FillOptions::default()),
        Transform::from_xyz(new_pos_x, new_pos_y, 0.0),
      ))
      .insert(SnakeBody {index: body_parts as u32})
      .insert(Velocity(Vec2::ZERO));
  }
}