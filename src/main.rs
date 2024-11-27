use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};

mod character;
use self::character::Character;

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
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

        Character::default()
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
