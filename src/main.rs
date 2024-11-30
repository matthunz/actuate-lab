use actuate::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};

mod character;
use self::character::{Character, CharacterState};

mod queue;
use self::queue::{use_queue_provider, use_queued};

mod skill;
use self::skill::IceShard;

#[derive(Data)]
pub struct Ui<'a> {
    character_states: SignalMut<'a, Vec<CharacterState>>,
    player_idx: usize,
    target_idx: usize,
}

impl Compose for Ui<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let entity = use_context::<Entity>(&cx).unwrap();

        let turn = use_mut(&cx, || 0);

        let font = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("C&C Red Alert [INET].ttf")
        });

        let is_turn_done = use_mut(&cx, || false);
        let on_click = use_queued(&cx, move || async move {
            SignalMut::update(turn, |turn| *turn += 1);
            SignalMut::set(is_turn_done, false);
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
        .target(*entity)
        .content(
            spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PickingBehavior::IGNORE,
            ))
            .content((
                spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ))
                .content((
                    IceShard {
                        character_states: cx.me().character_states,
                        player_idx: cx.me().player_idx,
                        target_idx: cx.me().target_idx,
                        turn: *turn,
                        is_turn_done: *is_turn_done,
                    },
                    IceShard {
                        character_states: cx.me().character_states,
                        player_idx: cx.me().player_idx,
                        target_idx: cx.me().target_idx,
                        turn: *turn,
                        is_turn_done: *is_turn_done,
                    },
                )),
                spawn((
                    Text::new("End Turn"),
                    TextColor(if *is_turn_done {
                        Color::srgb_u8(117, 117, 117)
                    } else {
                        Color::WHITE
                    }),
                    TextFont {
                        font: font.clone(),
                        font_size: 2.,
                        ..default()
                    },
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                ))
                .observe(move |_: In<Trigger<Pointer<Click>>>| {
                    SignalMut::set(is_turn_done, true);
                    on_click.queue();
                }),
            )),
        )
    }
}

#[derive(Data, Default)]
struct Game;

impl Compose for Game {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let target = use_mut(&cx, || 0);

        let entity = *use_world_once(&cx, |mut commands: Commands| {
            commands.spawn(Node::default()).id()
        });
        use_provider(&cx, || entity);

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
