use crate::assets::{AssetLoadingState, MyAssetPlugin, TileAssets};
use bevy::{math::Vec3Swizzles, platform::collections::HashSet, prelude::*};
use bevy_ecs_tilemap::{FrustumCulling, prelude::*};

mod assets;
mod noise;

// Press WASD to move the camera around, and watch as chunks spawn/despawn in response.

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16., y: 16. };

const CHUNK_SIZE: UVec2 = UVec2::splat(8);

const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

const DESPAWN_CHUNK_RADIUS: f32 = 640.;

// This is the speed for camera movement
const CAMERA_MOVEMENT_SPEED: f32 = 100.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_plugins(MyAssetPlugin)
        .insert_resource(ChunkManager::default())
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                spawn_chunks_around_camera,
                despawn_outofrange_chunks,
                move_camera,
            )
                .run_if(in_state(AssetLoadingState::Next)),
        )
        .run();
}

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

fn move_camera(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let delta_secs = time.delta_secs();

    let up = input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;

    for mut transform in camera_query.iter_mut() {
        transform.translation.x += horizontal as f32 * CAMERA_MOVEMENT_SPEED * delta_secs;
        transform.translation.y += vertical as f32 * CAMERA_MOVEMENT_SPEED * delta_secs;
    }
}

fn spawn_chunk(commands: &mut Commands, tile_assets: &TileAssets, chunk_pos: IVec2) {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(0),
                    ..Default::default()
                })
                .id();
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
        0.0,
    ));
    let texture_handle: Handle<Image> = tile_assets.tileset_image.clone();
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TILE_SIZE.into(),
        size: CHUNK_SIZE.into(),
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TILE_SIZE,
        transform,
        render_settings: TilemapRenderSettings {
            render_chunk_size: RENDER_CHUNK_SIZE,
            ..Default::default()
        },
        frustum_culling: FrustumCulling(true),
        ..Default::default()
    });
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    tile_assets: Res<TileAssets>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy());
        for y in (camera_chunk_pos.y - 2)..(camera_chunk_pos.y + 2) {
            for x in (camera_chunk_pos.x - 2)..(camera_chunk_pos.x + 2) {
                if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                    chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                    spawn_chunk(&mut commands, &tile_assets, IVec2::new(x, y));
                }
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation.xy().distance(chunk_pos);

            if distance > DESPAWN_CHUNK_RADIUS {
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn();
            }
        }
    }
}
