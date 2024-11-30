use super::Skill;
use crate::character::CharacterState;
use actuate::prelude::*;
use bevy::prelude::*;
use futures::future;
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

#[derive(Data)]
pub struct IceShard<'a> {
    pub character_states: SignalMut<'a, Vec<CharacterState>>,
    pub player_idx: usize,
    pub target_idx: usize,
    pub turn: u32,
    pub is_turn_done: bool,
}

impl Compose for IceShard<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        Skill {
            name: Cow::Owned(String::from("Ice Shard")),
            description: Cow::Owned(String::from(
                "Launch a shard of ice at the target, dealing 50 damage.",
            )),
            cooldown: 2,
            turn: cx.me().turn,
            is_enabled: !cx.me().is_turn_done,
            on_click: Box::new(move || {
                Box::pin(async move {
                    let duration = Duration::from_millis(500);
                    let arm = 0.5;
                    let leg = 0.25;

                    let character = cx.me().character_states[cx.me().player_idx].clone();

                    future::join(
                        character
                            .translation
                            .animate(Vec3::new(0., 0., -10.), Duration::from_millis(1500)),
                        async {
                            future::join4(
                                character.left_arm.animate(arm, duration),
                                character.right_arm.animate(-arm, duration),
                                character.left_leg.animate(leg, duration),
                                character.right_leg.animate(-leg, duration),
                            )
                            .await;

                            future::join4(
                                character.left_arm.animate(-arm, duration),
                                character.right_arm.animate(arm, duration),
                                character.left_leg.animate(-leg, duration),
                                character.right_leg.animate(leg, duration),
                            )
                            .await;

                            future::join4(
                                character.left_arm.animate(0., duration),
                                character.right_arm.animate(0., duration),
                                character.left_leg.animate(0., duration),
                                character.right_leg.animate(0., duration),
                            )
                            .await;

                            character
                                .right_arm
                                .animate(FRAC_PI_2, Duration::from_millis(200))
                                .await;

                            let player_idx = cx.me().player_idx;
                            let target_idx = cx.me().target_idx;
                            SignalMut::update(cx.me().character_states, move |characters| {
                                let character_mut = &mut characters[player_idx];
                                character_mut.energy -= 1;

                                let target_character = &mut characters[target_idx];
                                target_character.health -= 1;
                            });

                            character
                                .right_arm
                                .animate(0., Duration::from_millis(200))
                                .await;

                            character
                                .rotation
                                .animate(Vec3::new(0., PI, 0.), Duration::from_millis(250))
                                .await;
                        },
                    )
                    .await;

                    future::join(
                        character
                            .translation
                            .animate(Vec3::new(0., 0., 40.), Duration::from_millis(1500)),
                        async {
                            future::join4(
                                character.left_arm.animate(arm, duration),
                                character.right_arm.animate(-arm, duration),
                                character.left_leg.animate(leg, duration),
                                character.right_leg.animate(-leg, duration),
                            )
                            .await;

                            future::join4(
                                character.left_arm.animate(-arm, duration),
                                character.right_arm.animate(arm, duration),
                                character.left_leg.animate(-leg, duration),
                                character.right_leg.animate(leg, duration),
                            )
                            .await;

                            future::join4(
                                character.left_arm.animate(0., duration),
                                character.right_arm.animate(0., duration),
                                character.left_leg.animate(0., duration),
                                character.right_leg.animate(0., duration),
                            )
                            .await;
                            character
                                .rotation
                                .animate(Vec3::new(0., 0., 0.), Duration::from_millis(250))
                                .await;
                        },
                    )
                    .await;
                })
            }),
        }
    }
}
