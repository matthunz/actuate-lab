use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use std::time::Duration;

mod character;
use self::character::Character;

mod queue;
use self::queue::{use_queue_provider, use_queued};

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
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

        let x = use_animated(&cx, || 0.);

        let task = use_queued(&cx, move || async move {
            loop {
                x.animate(1., Duration::from_secs(1)).await;
                x.animate(0., Duration::from_secs(1)).await;
            }
        });

        Character {
            left_arm_rotation: *x,
            ..default()
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Composition::new(Game));
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ActuatePlugin, voxy::DefaultPlugins))
        .add_systems(Startup, setup)
        .run();
}
