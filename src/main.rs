// #![allow(unused)]

mod enemy;
mod lib;
mod player;

use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use enemy::EnemyPlugin;
use lib::texture;
use player::PlayerPlugin;
use std::collections::HashSet;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const TIME_STEP: f32 = 1. / 60.;
const SCALE: f32 = 0.5;
const MAX_ENEMIES: u32 = 4;
const MAX_FORMATION_MEMBERS: u32 = 2;
// TODO: make dependent on monitor resolution
const SPEED_MULTIPLIER: f32 = 1.;
const PLAYER_RESPAWN_DELAY: f64 = 2.; // 2s

// Entity, Component, System, Resource

// region: resource
pub struct Materials {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

struct WinSize {
    #[allow(unused)]
    w: f32,
    h: f32,
}

struct ActiveEnemies(u32);
struct PlayerState {
    on: bool,
    last_shot: f64,
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: 0.,
        }
    }
}
impl PlayerState {
    fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time
    }

    fn spawned(&mut self) {
        self.on = true;
        self.last_shot = 0.
    }
}

// endregion: resource

// region: component

#[derive(Component)]
struct Laser;

#[derive(Component)]
struct Player;
#[derive(Component)]
struct PlayerReadyFire(bool);
#[derive(Component)]
struct FromPlayer;

#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct FromEnemy;

#[derive(Component)]
struct Explosion;
#[derive(Component)]
struct ExplosionToSpawn(Vec3);

#[derive(Component)]
struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self(500. * SPEED_MULTIPLIER)
    }
}

// endregion: component

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".into(),
            width: 640.0,
            height: 800.0,
            ..Default::default()
        })
        .insert_resource(ActiveEnemies(0))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
        .run()
}

fn setup(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary_mut().unwrap();

    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // create the main resources
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    commands.insert_resource(Materials {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion: texture_atlases.add(texture_atlas),
    });
    commands.insert_resource(WinSize {
        w: window.width(),
        h: window.height(),
    });
}

fn player_laser_hit_enemy(
    mut commands: Commands,
    laser_query: Query<
        (Entity, &Transform, Option<&Handle<Image>>),
        (With<Laser>, With<FromPlayer>),
    >,
    enemy_query: Query<(Entity, &Transform, Option<&Handle<Image>>), With<Enemy>>,
    mut active_enemies: ResMut<ActiveEnemies>,
    images: Res<Assets<Image>>,
) {
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();

    for (laser_entity, laser_tf, laser_opt_handle) in laser_query.iter() {
        for (enemy_entity, enemy_tf, enemy_opt_handle) in enemy_query.iter() {
            let laser_scale = Vec3::from(laser_tf.scale);
            let enemy_scale = Vec3::from(enemy_tf.scale);

            let laser_size = texture::get_size(laser_opt_handle, &images);
            let enemy_size = texture::get_size(enemy_opt_handle, &images);

            let collision = collide(
                laser_tf.translation,
                laser_size * laser_scale.truncate().abs(),
                enemy_tf.translation,
                enemy_size * enemy_scale.truncate().abs(),
            );

            if let Some(_) = collision {
                info!("Active Enemies: {}", active_enemies.0);

                if enemies_blasted.get(&enemy_entity).is_none() {
                    // remove enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;

                    // spawn explosion
                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));

                    enemies_blasted.insert(enemy_entity);
                }

                // remove the laser
                commands.entity(laser_entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<
        (Entity, &Transform, Option<&Handle<Image>>),
        (With<Laser>, With<FromEnemy>),
    >,
    player_query: Query<(Entity, &Transform, Option<&Handle<Image>>), With<Player>>,
    images: Res<Assets<Image>>,
) {
    for (laser_entity, laser_tf, laser_opt_handle) in laser_query.iter() {
        for (player_entity, player_tf, player_opt_handle) in player_query.iter() {
            let laser_scale = Vec3::from(laser_tf.scale);
            let player_scale = Vec3::from(player_tf.scale);

            let laser_size = texture::get_size(laser_opt_handle, &images);
            let player_size = texture::get_size(player_opt_handle, &images);

            let collision = collide(
                laser_tf.translation,
                laser_size * laser_scale.truncate().abs(),
                player_tf.translation,
                player_size * player_scale.truncate().abs(),
            );

            if let Some(_) = collision {
                // remove player
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());

                // remove laser
                commands.entity(laser_entity).despawn();

                // spawn explosion
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: materials.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<
        (
            Entity,
            &mut Timer,
            &mut TextureAtlasSprite,
            &Handle<TextureAtlas>,
        ),
        With<Explosion>,
    >,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;

            if sprite.index == texture_atlas.textures.len() as usize {
                commands.entity(entity).despawn();
            }
        }
    }
}
