mod assets;

use bevy::{
    camera::{Projection::Orthographic, ScalingMode},
    pbr::MeshBatchSetCompareData,
    prelude::*,
    sprite_render::Mesh2dWireframeTemplate,
};

use crate::assets::{AssetLoadingState, MyAssetPlugin, TileAssets};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MyAssetPlugin)
        .add_systems(OnEnter(AssetLoadingState::Next), spawn_camera)
        .add_systems(
            Update,
            (move_camera, spawn_and_despawn_tiles).run_if(in_state(AssetLoadingState::Next)),
        )
        .run();
}

#[derive(Component)]
struct Tile;

fn spawn_camera(mut command: Commands) {
    command.spawn((
        Camera2d,
        Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 200.,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn move_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let delta_secs = time.delta_secs();

    for mut transform in query.iter_mut() {
        let up = input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
        let down = input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
        let left = input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
        let right = input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

        let direction_x = right as i8 - left as i8;
        let direction_y = up as i8 - down as i8;

        transform.translation.x += direction_x as f32 * 20. * delta_secs;
        transform.translation.y += direction_y as f32 * 20. * delta_secs;
    }
}

fn spawn_and_despawn_tiles(
    mut command: Commands,
    tile_assets: Res<TileAssets>,
    camera_transform_query: Query<&Transform, (With<Camera>, Without<Tile>)>,
    tile_query: Query<(Entity, &Transform), (With<Tile>, Without<Camera>)>,
) {
    for camera_transform in camera_transform_query {
        let new_tile_pos_x = ((camera_transform.translation.x / 16.).floor() * 16.) + 8.;
        let new_tile_pos_y = ((camera_transform.translation.y / 16.).floor() * 16.) + 8.;

        let mut should_spawn_tile = true;
        for (entity, tile_transform) in tile_query {
            if tile_transform.translation.x == new_tile_pos_x
                && tile_transform.translation.y == new_tile_pos_y
            {
                should_spawn_tile = false;
            } else {
                command.entity(entity).despawn();
            }
        }

        if should_spawn_tile {
            println!("Spawning tile at: {new_tile_pos_x}:{new_tile_pos_y}");
            println!(
                "Current camera position: {}:{}",
                camera_transform.translation.x, camera_transform.translation.y,
            );

            command.spawn((
                Tile,
                Sprite::from_atlas_image(
                    tile_assets.tileset_image.clone(),
                    TextureAtlas {
                        layout: tile_assets.atlas_layout.clone(),
                        index: 0,
                    },
                ),
                Transform::from_translation(vec3(new_tile_pos_x, new_tile_pos_y, 0.)),
            ));
        }
    }
}
