

//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

use std::f32::consts::{FRAC_PI_4, PI};
use bevy_web_asset::WebAssetPlugin;
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::pbr::extract_meshes;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::time::FixedTimestep;

const TIME_STEP: f32 = 1.0 / 60.0;

const PADDLE_SIZE: Vec3 = Vec3::new(2.0, 1.0, 1.0);
const BRICK_SIZE: Vec3 = Vec3::new(1.0, 0.4, 1.0);
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 0.5;
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 0.2;
const GAP_BETWEEN_BRICKS: f32 = 0.3;
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 0.3;
const PADDLE_PADDING: f32 = 0.1;
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 3.0;

const BALL_SIZE: Vec3 = Vec3::new(0.2, 0.2, 0.2);
const BALL_SPEED: f32 = 7.0;
const PADDLE_SPEED: f32 = 8.0;
const WALL_THICKNESS: f32 = 1.0;
const BALL_STARTING_POSITION: Vec3 = Vec3::new(-4.0, 2.0, 0.0);
const INITIAL_BALL_DIRECTION: Vec3 = Vec3::new(0.5, -0.5, 0.0);
// Wall coordinates
const LEFT_WALL: f32 = 0.0;
const RIGHT_WALL: f32 = 10.0;
const TOP_WALL: f32 = 10.0;
const BOTTOM_WALL: f32 = 0.0;


const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

// Colors
const WALL_COLOR: Color = Color::rgb(0., 0., 0.);
const BRICK_COLOR: Color = Color::rgb(0., 0., 0.);
const SCORE_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Bundle)]
struct WallBundle {
    pbr_bundle: PbrBundle,
    collider: Collider,
}

// Which side of the arena is the wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec3 {
        match self {
            WallLocation::Left => Vec3::new(LEFT_WALL - (RIGHT_WALL / 2.0), TOP_WALL / 2.0,0.),
            WallLocation::Right => Vec3::new(RIGHT_WALL / 2., TOP_WALL / 2.0, 0.),
            WallLocation::Top => Vec3::new(0.0, TOP_WALL, 0.),
            WallLocation::Bottom => Vec3::new(0.0,BOTTOM_WALL,0.),
        }
    }

    fn size(&self) -> Vec3 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        //assert that constants have legal values
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);
        println!("{}. {}", arena_height, arena_width);
        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec3::new(WALL_THICKNESS, arena_height + WALL_THICKNESS , 1.)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec3::new(arena_height + WALL_THICKNESS, WALL_THICKNESS, 1.)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation, material: Handle<StandardMaterial>, mesh: Handle<Mesh>) -> WallBundle {
        println!("{}", location.position().to_string());
         return WallBundle {
                    pbr_bundle: PbrBundle {
                    transform: Transform::from_translation(location.position()).with_scale(location.size()),
                    material,
                    mesh,
                        ..default()
                    },
                    collider: Collider,
        }
    }

}

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

fn main() {
    App::new()
        .insert_resource(Scoreboard { score: 0})
        .insert_resource(ClearColor(Color::rgb(0.7, 1.0, 1.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(rotate)
                .with_system(check_for_collision)
                .with_system(move_object.before(check_for_collision))
                .with_system(apply_velocity.before(check_for_collision))
        )
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: Res<AssetServer>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 50. }.into()),
        material: materials.add(Color::SILVER.into()),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });

    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 25.0, 10.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10., 20.0).looking_at(Vec3::new(0., 5., 0.), Vec3::Y),
        ..default()
    });
    // ball
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::UVSphere::default().into()).into(),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..default()

            }),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE)
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize()*BALL_SPEED),
    ));
    let wall_material = materials.add(StandardMaterial{
        base_color: Color::PURPLE,
        ..default()
    });
    let wall_mesh: Handle<Mesh> = meshes.add(shape::Cube::default().into()).into();
    commands.spawn(WallBundle::new(WallLocation::Left, wall_material.clone(), wall_mesh.clone()));
    commands.spawn(WallBundle::new(WallLocation::Right, wall_material.clone(), wall_mesh.clone()));
    commands.spawn(WallBundle::new(WallLocation::Bottom, wall_material.clone(), wall_mesh.clone()));
    commands.spawn(WallBundle::new(WallLocation::Top, wall_material.clone(), wall_mesh.clone()));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::default().into()).into(),
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0., 2.0, 0.)).with_scale(Vec3::new(1.0, 0.2, 1.0)),
            ..default()
        },
        Paddle,
        Collider,
        ));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
            }),
        ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            }),
    );

    assert!(BRICK_SIZE.x > 0.0);
    assert!(BRICK_SIZE.y > 0.0);
    assert!(BRICK_SIZE.z > 0.0);
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = 0.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::Cube::default().into()).into(),
                    material: debug_material.clone(),
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn rotate(mut query: Query<&mut Transform, With<Paddle>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn move_object(mut query: Query<&mut Transform, With<Paddle>>, keyboard_input: Res<Input<KeyCode>>){
    let mut direction = 0.0;
    let mut object_transform = query.single_mut();
    if keyboard_input.pressed(KeyCode::Up) {
        direction += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down){
        direction -= 1.0;
    }

    let new_object_positiion = object_transform.translation.x + direction * PADDLE_SPEED * TIME_STEP;

    let left_bound = -5.0 + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = 5.0 - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    object_transform.translation.x = new_object_positiion.clamp(left_bound, right_bound);
}
/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
        transform.translation.z += velocity.z * TIME_STEP;
    }
}

fn check_for_collision(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {

    let (mut ball_velocity, ball_transform)  = ball_query.single_mut();
    for (collider_entity, transform, maybe_brick) in &collider_query {
        let collision = collide(
            ball_transform.translation,
            ball_transform.scale.truncate(),
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(collision) = collision {

            collision_events.send_default();
            if maybe_brick.is_some() {

                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => { /* do nothing */ }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }


        }
}