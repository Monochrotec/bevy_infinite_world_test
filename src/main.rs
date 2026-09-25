use crate::{
    assets::{AssetLoadingState, MyAssetPlugin, TileAssets},
    noise::{Noise1d, Noises},
};
use bevy::{camera::ScalingMode, math::Vec3Swizzles, platform::collections::HashMap, prelude::*};
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

const CHUNK_LOAD_SIZE: UVec2 = UVec2::splat(2);

// This is the speed for camera movement
const CAMERA_MOVEMENT_SPEED: f32 = 100.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TilemapPlugin)
        .add_plugins(MyAssetPlugin)
        .insert_resource(ChunkManager::default())
        .insert_resource(Noises {
            terrain: Noise1d::new(0),
        })
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
        .add_observer(spawn_chunk)
        .add_observer(despawn_chunk)
        .run();
}

#[derive(Default, Debug, Clone, Resource)]
struct ChunkManager {
    pub spawned_chunks: HashMap<IVec2, Entity>,
}

impl ChunkManager {
    pub fn is_there_chunk_here(&self, x: i32, y: i32) -> bool {
        self.spawned_chunks.contains_key(&IVec2::new(x, y))
    }
}

#[derive(Event)]
struct SpawnChunk(IVec2);

#[derive(Event)]
struct DespawnChunk(IVec2);

fn spawn_chunk(
    spawn_chunk_event: On<SpawnChunk>,
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    tile_assets: Res<TileAssets>,
    mut noises: ResMut<Noises>,
) {
    let chunk_pos = spawn_chunk_event.0;

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
    // Spawn the elements of the tilemap.
    for x in 0..CHUNK_SIZE.x {
        let terrain_height = (noises.terrain.get(x as f64 / 100.).max(0.) * 10.).round() as i32;

        for y in 0..CHUNK_SIZE.y {
            let absolute_tile_location_y =
                y as i32 + (chunk_pos.y - 1) * TILE_SIZE.y.round() as i32;
            if terrain_height >= absolute_tile_location_y {
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

    chunk_manager
        .spawned_chunks
        .insert(chunk_pos, tilemap_entity);

    println!("Spawned chunk at {chunk_pos}");
}

fn despawn_chunk(
    despawn_chunk_event: On<DespawnChunk>,
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let chunk_pos = despawn_chunk_event.0;
    if let Some(entity) = chunk_manager.spawned_chunks.remove(&chunk_pos) {
        commands.entity(entity).despawn();
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 400.,
            },
            scale: 1.,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    chunk_manager: Res<ChunkManager>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy());
        for y in (camera_chunk_pos.y - CHUNK_LOAD_SIZE.y as i32)
            ..(camera_chunk_pos.y + CHUNK_LOAD_SIZE.y as i32)
        {
            for x in (camera_chunk_pos.x - CHUNK_LOAD_SIZE.x as i32)
                ..(camera_chunk_pos.x + CHUNK_LOAD_SIZE.x as i32)
            {
                if !chunk_manager.is_there_chunk_here(x, y) {
                    commands.trigger(SpawnChunk(IVec2::new(x, y)));
                }
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunk_manager: Res<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy());

        for pos in chunk_manager.spawned_chunks.keys() {
            if !(((camera_chunk_pos.y - CHUNK_LOAD_SIZE.y as i32)
                ..(camera_chunk_pos.y + CHUNK_LOAD_SIZE.y as i32))
                .contains(&pos.y)
                && ((camera_chunk_pos.x - CHUNK_LOAD_SIZE.x as i32)
                    ..(camera_chunk_pos.x + CHUNK_LOAD_SIZE.x as i32))
                    .contains(&pos.x))
            {
                commands.trigger(DespawnChunk(*pos));
            }
        }
    }
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
