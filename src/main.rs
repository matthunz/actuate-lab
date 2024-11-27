use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use futures::future;
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

mod character;
use self::character::{Character, CharacterState};

mod queue;
use self::queue::{use_queue_provider, use_queued};

mod skill;
use self::skill::Skill;

#[derive(Data)]
struct IceShard<'a> {
    character_states: SignalMut<'a, Vec<CharacterState>>,
    player_idx: usize,
    target_idx: usize,
}

impl Compose for IceShard<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        Skill {
            name: Cow::Owned(String::from("Ice Shard")),
            description: Cow::Owned(String::from(
                "Launch a shard of ice at the target, dealing 50 damage.",
            )),
            cooldown: 2,
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

#[derive(Data)]
pub struct Ui<'a> {
    character_states: SignalMut<'a, Vec<CharacterState>>,
    player_idx: usize,
    target_idx: usize,
}

impl Compose for Ui<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let entity = *use_world_once(&cx, |mut commands: Commands| {
            commands.spawn(Node::default()).id()
        });

        spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                flex_grow: 1.,
                align_self: AlignSelf::Stretch,
                justify_self: JustifySelf::Stretch,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .target(entity)
        .content(IceShard {
            character_states: cx.me().character_states,
            player_idx: cx.me().player_idx,
            target_idx: cx.me().target_idx,
        })
    }
}

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let target = use_mut(&cx, || 0);

        use_queue_provider(&cx);

        use_world_once(&cx, |mut commands: Commands| {
            commands.spawn((
                Camera {
                    hdr: true,
                    ..default()
                },
                Camera3d::default(),
                Transform::from_translation(Vec3::new(0., 200., 100.))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Bloom::NATURAL,
            ));
        });

        let character_states = use_mut(&cx, Vec::new);

        (
            Character {
                index: 0,
                target: *target,
                transation: Vec3::new(0., 0., 40.),
                on_mount: Box::new(move |state| {
                    SignalMut::update(character_states, move |states| states.push(state));
                }),
                on_click: Box::new(move || SignalMut::set(target, 0)),
                health: character_states
                    .get(0)
                    .map(|state| state.health)
                    .unwrap_or(100),
                energy: character_states
                    .get(0)
                    .map(|state| state.energy)
                    .unwrap_or(10),
            },
            Character {
                index: 1,
                target: *target,
                transation: Vec3::new(0., 0., -40.),
                on_mount: Box::new(move |state| {
                    SignalMut::update(character_states, move |states| states.push(state));
                }),
                on_click: Box::new(move || SignalMut::set(target, 1)),
                health: character_states
                    .get(1)
                    .map(|state| state.health)
                    .unwrap_or(100),
                energy: character_states
                    .get(1)
                    .map(|state| state.energy)
                    .unwrap_or(10),
            },
            Ui {
                character_states,
                player_idx: 0,
                target_idx: *target,
            },
        )
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Composition::new(Game));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MeshPickingPlugin,
            ActuatePlugin,
            voxy::DefaultPlugins,
        ))
        .add_systems(Startup, setup)
        .insert_resource(UiScale(20.))
        .run();
}
