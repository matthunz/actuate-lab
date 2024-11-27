use std::{future::Future, pin::Pin};

use crate::use_queued;
use actuate::prelude::*;
use bevy::prelude::*;

#[derive(Data)]
pub struct Skill<'a> {
    pub name: Cow<'a, String>,
    pub description: Cow<'a, String>,
    pub on_click: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + 'a>> + 'a>,
}

impl Compose for Skill<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let frame = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load::<Image>("frame.png")
        });
        let icicle = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load::<Image>("icicle.png")
        });

        let is_hovered = use_mut(&cx, || false);
        let is_pointer_down = use_mut(&cx, || false);

        let task = use_queued(&cx, move || (cx.me().on_click)());

        spawn((Node {
            width: Val::Px(4.),
            height: Val::Px(4.),
            ..default()
        },))
        .content((
            spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.25),
                    left: Val::Px(0.25),
                    width: Val::Px(3.5),
                    height: Val::Px(3.5),
                    ..default()
                },
                UiImage::new(icicle.clone()),
                ZIndex(3),
                PickingBehavior::IGNORE,
            )),
            spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.),
                    left: Val::Px(0.),
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                UiImage::new(frame.clone()).with_color(if *is_pointer_down {
                    Color::WHITE
                } else {
                    Color::BLACK
                }),
                ZIndex(2),
            ))
            .observe(move |_trigger: In<Trigger<Pointer<Over>>>| {
                SignalMut::set_if_neq(is_hovered, true)
            })
            .observe(move |_trigger: In<Trigger<Pointer<Out>>>| {
                SignalMut::set(is_hovered, false);
                SignalMut::set(is_pointer_down, false);
            })
            .observe(move |_trigger: In<Trigger<Pointer<Down>>>| {
                SignalMut::set(is_pointer_down, true)
            })
            .observe(move |_trigger: In<Trigger<Pointer<Up>>>| {
                SignalMut::set(is_pointer_down, false)
            })
            .observe(move |_trigger: In<Trigger<Pointer<Click>>>| task.queue()),
            if *is_hovered {
                Some(Menu {
                    name: Signal::map(cx.me(), |me| &*me.name).into(),
                    description: Signal::map(cx.me(), |me| &*me.description).into(),
                })
            } else {
                None
            },
        ))
    }
}

#[derive(Data)]
struct Menu<'a> {
    name: Cow<'a, String>,
    description: Cow<'a, String>,
}

impl Compose for Menu<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let menu = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load::<Image>("menu.png")
        });

        let font = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("C&C Red Alert [INET].ttf")
        });

        let slicer = TextureSlicer {
            border: BorderRect::square(6.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        };

        let entity_cell = use_mut(&cx, || None);

        let left = use_mut(&cx, || None);

        use_world(
            &cx,
            move |query: Query<&Parent>, layout_query: Query<&ComputedNode>| {
                if let Some(entity) = *entity_cell {
                    let node = layout_query.get(entity).unwrap();

                    let parent_entity = query.get(entity).unwrap();
                    let parent_node = layout_query.get(parent_entity.get()).unwrap();

                    SignalMut::set_if_neq(
                        left,
                        Some(-(node.size().x / 2. - parent_node.size().x / 2.)),
                    );
                }
            },
        );

        spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left.unwrap_or_default()),
                bottom: Val::Px(4.),
                width: Val::Vw(100.),
                max_width: Val::Px(20.),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(2.)),
                ..default()
            },
            if left.is_some() {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            PickingBehavior::IGNORE,
        ))
        .on_spawn(move |entity| {
            SignalMut::set(entity_cell, Some(entity.id()));
        })
        .content((
            spawn((
                Text::new(cx.me().name.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 2.,
                    ..default()
                },
                ZIndex(1),
                PickingBehavior::IGNORE,
            )),
            spawn((
                Text::new(cx.me().description.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 1.,
                    ..default()
                },
                ZIndex(1),
                PickingBehavior::IGNORE,
            )),
            spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.),
                    left: Val::Px(0.),
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                UiImage::new(menu.clone()).with_mode(NodeImageMode::Sliced(slicer)),
                PickingBehavior::IGNORE,
            )),
        ))
    }
}
