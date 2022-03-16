use bevy::prelude::*;

use crate::{Materials, SCALE, TIME_STEP, WinSize, Speed, Player, PlayerReadyFire, Laser, FromPlayer};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
       app
        .add_startup_stage("game_setup_actors", SystemStage::single(player_spawn.system()))
        .add_system(player_movement.system())
        .add_system(player_fire.system())
        .add_system(laser_movement.system());
    }
}

fn player_spawn (
    mut commands: Commands,
    materials: Res<Materials>,
    win_size: Res<WinSize>
) {
    // spawn sprite
    let bottom = -win_size.h / 2.;
    commands
        .spawn_bundle(SpriteBundle {
            texture: materials.player.clone(),
            transform: Transform {
                translation: Vec3::new(0., bottom + 75. / 4. + 5., 10.),
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(PlayerReadyFire(true))
        .insert(Speed::default());
}

fn player_movement (
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Transform), With<Player>>
) {
    if let Ok((speed, mut transform)) = query.get_single_mut() {
        let dir = if keyboard_input.pressed(KeyCode::Left) {
            -1.
        } else if keyboard_input.pressed(KeyCode::Right) {
            1.
        } else {
            0.
        };

        transform.translation.x += dir * speed.0 * TIME_STEP;
    }
}

fn player_fire (
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    materials: Res<Materials>,
    mut query: Query<(&Transform, &mut PlayerReadyFire), With<Player>>
) {
    if let Ok((player_tf, mut ready_fire)) = query.get_single_mut() {
        if ready_fire.0 && kb.pressed(KeyCode::Space) {
            let x = player_tf.translation.x;
            let y = player_tf.translation.y;

            let mut spawn_lasers = |x_offset: f32| {
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: materials.player_laser.clone(),
                        transform: Transform {
                            translation: Vec3::new(x + x_offset, y + 15., 0.),
                            scale: Vec3::new(SCALE, SCALE, 1.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(Speed::default());
            };

            let x_offset = 144.0 / 4.0 - 5.0;
            spawn_lasers(x_offset);
            spawn_lasers(-x_offset);

            ready_fire.0 = false;
        }

        if kb.just_released(KeyCode::Space) {
            ready_fire.0 = true;
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<FromPlayer>)>
) {
    for (laser_entity, speed, mut laser_tf) in query.iter_mut() {
        let translation = &mut laser_tf.translation;
        translation.y += speed.0 * TIME_STEP;
        if translation.y > win_size.h {
            commands.entity(laser_entity).despawn();
        }
    }
}
