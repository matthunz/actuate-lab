use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use futures::future;
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

mod character;
use self::character::Character;

mod queue;
use self::queue::{use_queue_provider, use_queued};

mod skill;
use self::skill::Skill;

#[derive(Clone, Copy, Data)]
struct CharacterData<'a> {
    translation: UseAnimated<'a, Vec3>,
    rotation: UseAnimated<'a, Vec3>,
    left_arm: UseAnimated<'a, f32>,
    right_arm: UseAnimated<'a, f32>,
    left_leg: UseAnimated<'a, f32>,
    right_leg: UseAnimated<'a, f32>,
}

#[derive(Data)]
struct IceShard<'a> {
    character: CharacterData<'a>,
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

                    future::join(
                        cx.me()
                            .character
                            .translation
                            .animate(Vec3::new(0., 0., -10.), Duration::from_millis(1500)),
                        async {
                            future::join4(
                                cx.me().character.left_arm.animate(arm, duration),
                                cx.me().character.right_arm.animate(-arm, duration),
                                cx.me().character.left_leg.animate(leg, duration),
                                cx.me().character.right_leg.animate(-leg, duration),
                            )
                            .await;

                            future::join4(
                                cx.me().character.left_arm.animate(-arm, duration),
                                cx.me().character.right_arm.animate(arm, duration),
                                cx.me().character.left_leg.animate(-leg, duration),
                                cx.me().character.right_leg.animate(leg, duration),
                            )
                            .await;

                            future::join4(
                                cx.me().character.left_arm.animate(0., duration),
                                cx.me().character.right_arm.animate(0., duration),
                                cx.me().character.left_leg.animate(0., duration),
                                cx.me().character.right_leg.animate(0., duration),
                            )
                            .await;

                            cx.me()
                                .character
                                .right_arm
                                .animate(FRAC_PI_2, Duration::from_millis(200))
                                .await;
                            cx.me()
                                .character
                                .right_arm
                                .animate(0., Duration::from_millis(200))
                                .await;

                            cx.me()
                                .character
                                .rotation
                                .animate(Vec3::new(0., PI, 0.), Duration::from_millis(250))
                                .await;
                        },
                    )
                    .await;

                    future::join(
                        cx.me()
                            .character
                            .translation
                            .animate(Vec3::new(0., 0., 40.), Duration::from_millis(1500)),
                        async {
                            future::join4(
                                cx.me().character.left_arm.animate(arm, duration),
                                cx.me().character.right_arm.animate(-arm, duration),
                                cx.me().character.left_leg.animate(leg, duration),
                                cx.me().character.right_leg.animate(-leg, duration),
                            )
                            .await;

                            future::join4(
                                cx.me().character.left_arm.animate(-arm, duration),
                                cx.me().character.right_arm.animate(arm, duration),
                                cx.me().character.left_leg.animate(-leg, duration),
                                cx.me().character.right_leg.animate(leg, duration),
                            )
                            .await;

                            future::join4(
                                cx.me().character.left_arm.animate(0., duration),
                                cx.me().character.right_arm.animate(0., duration),
                                cx.me().character.left_leg.animate(0., duration),
                                cx.me().character.right_leg.animate(0., duration),
                            )
                            .await;

                            cx.me()
                                .character
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
    character: CharacterData<'a>,
}

impl Compose for Ui<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let entity = *use_world_once(&cx, |mut commands: Commands| {
            commands.spawn(Node::default()).id()
        });

        spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::End,
            flex_grow: 1.,
            align_self: AlignSelf::Stretch,
            justify_self: JustifySelf::Stretch,
            ..default()
        })
        .target(entity)
        .content(IceShard {
            character: cx.me().character,
        })
    }
}

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let translation = use_animated(&cx, || Vec3::new(0., 0., 40.));
        let rotation = use_animated(&cx, || Vec3::ZERO);

        let left_arm = use_animated(&cx, || 0.);
        let right_arm = use_animated(&cx, || 0.);
        let left_leg = use_animated(&cx, || 0.);
        let right_leg = use_animated(&cx, || 0.);

        let character = CharacterData {
            translation,
            rotation,
            left_arm,
            right_arm,
            left_leg,
            right_leg,
        };

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

        (
            Character {
                transform: Transform::from_translation(*translation).with_rotation(
                    Quat::from_euler(EulerRot::YXZ, rotation.y, rotation.x, rotation.z),
                ),
                left_arm_rotation: *left_arm,
                right_arm_rotation: *right_arm,
                left_leg_rotation: *left_leg,
                right_leg_rotation: *right_leg,
            },
            Character {
                transform: Transform::from_xyz(0., 0., -40.)
                    .with_rotation(Quat::from_rotation_y(PI)),
                ..default()
            },
            Ui { character },
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
            ActuatePlugin,
            voxy::DefaultPlugins,
        ))
        .add_systems(Startup, setup)
        .insert_resource(UiScale(20.))
        .run();
}
