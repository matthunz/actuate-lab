use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use std::time::Duration;

mod character;
use self::character::Character;

mod queue;
use self::queue::{use_queue_provider, use_queued};

mod skill;
use self::skill::Skill;

#[derive(Data)]
struct IceShard<'a> {
    x: UseAnimated<'a, f32>,
}

impl Compose for IceShard<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        Skill {
            name: Cow::Owned(String::from("Ice Shard")),
            description: Cow::Owned(String::from(
                "Launch a shard of ice at the target, dealing 50 damage.",
            )),
            on_click: Box::new(move || {
                Box::pin(async move {
                    cx.me().x.animate(1., Duration::from_secs(1)).await;
                    cx.me().x.animate(0., Duration::from_secs(1)).await;
                })
            }),
        }
    }
}

#[derive(Data)]
pub struct Ui<'a> {
    x: UseAnimated<'a, f32>,
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
        .content(IceShard { x: cx.me().x })
    }
}

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let x = use_animated(&cx, || 0.);

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
                left_arm_rotation: *x,
                ..default()
            },
            Ui { x },
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
