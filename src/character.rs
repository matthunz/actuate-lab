use actuate::{animation::AnimationController, prelude::*};
use bevy::prelude::*;
use std::{
    cell::Cell,
    mem,
    sync::{Arc, Mutex},
};
use voxy::{scene::VoxelSceneHandle, VoxelSceneModels};

#[derive(Clone, Data)]
pub struct CharacterState {
    pub translation: AnimationController<Vec3>,
    pub rotation: AnimationController<Vec3>,
    pub left_arm: AnimationController<f32>,
    pub right_arm: AnimationController<f32>,
    pub left_leg: AnimationController<f32>,
    pub right_leg: AnimationController<f32>,
    pub health: u32,
    pub energy: u32,
}

#[derive(Data)]
pub struct Character<'a> {
    pub index: usize,
    pub target: usize,
    pub transation: Vec3,
    pub on_mount: Box<dyn Fn(CharacterState) + 'a>,
    pub on_click: Box<dyn Fn() + Send + Sync + 'a>,
    pub health: u32,
    pub energy: u32,
}

impl Compose for Character<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let translation = use_animated(&cx, || cx.me().transation);
        let rotation = use_animated(&cx, || Vec3::ZERO);

        let left_arm = use_animated(&cx, || 0.);
        let right_arm = use_animated(&cx, || 0.);
        let left_leg = use_animated(&cx, || 0.);
        let right_leg = use_animated(&cx, || 0.);

        use_ref(&cx, || {
            (cx.me().on_mount)(CharacterState {
                translation: translation.controller(),
                rotation: rotation.controller(),
                left_arm: left_arm.controller(),
                right_arm: right_arm.controller(),
                left_leg: left_leg.controller(),
                right_leg: right_leg.controller(),
                health: 100,
                energy: 10,
            })
        });

        let on_click = Signal::map(cx.me(), |me| &me.on_click);

        CharacterModel {
            transform: Transform::from_translation(*translation).with_rotation(Quat::from_euler(
                EulerRot::YXZ,
                rotation.y,
                rotation.x,
                rotation.z,
            )),
            left_arm_rotation: *left_arm,
            right_arm_rotation: *right_arm,
            left_leg_rotation: *left_leg,
            right_leg_rotation: *right_leg,
            health: cx.me().health,
            energy: cx.me().energy,
            is_selected: cx.me().target == cx.me().index,
            on_click: Box::new(move || (on_click)()),
        }
    }
}

#[derive(Data)]
pub struct CharacterModel<'a> {
    pub transform: Transform,
    pub left_arm_rotation: f32,
    pub right_arm_rotation: f32,
    pub left_leg_rotation: f32,
    pub right_leg_rotation: f32,
    pub health: u32,
    pub energy: u32,
    pub is_selected: bool,
    pub on_click: Box<dyn Fn() + Send + Sync + 'a>,
}

impl Compose for CharacterModel<'_> {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let handle = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("character.vox")
        });

        let entity = use_bundle(&cx, || {
            (
                VoxelSceneHandle(handle.clone()),
                cx.me().transform,
                Visibility::default(),
            )
        });

        let on_click: Map<'_, Box<dyn Fn() + Send + Sync>> =
            Signal::map(cx.me(), |me| &me.on_click);
        let on_click: Map<'_, Box<dyn Fn() + Send + Sync>> = unsafe { mem::transmute(on_click) };

        let is_loaded = use_ref(&cx, || Cell::new(false));

        let is_mouse_down = Arc::new(Mutex::new(false));

        use_world(
            &cx,
            move |mut commands: Commands, query: Query<&Children>| {
                let is_mouse_down = is_mouse_down.clone();

                if !is_loaded.get() {
                    for child in query.get(entity).into_iter().flatten() {
                        is_loaded.set(true);

                        let is_mouse_down = is_mouse_down.clone();
                        let is_mouse_down_out = is_mouse_down.clone();
                        let is_mouse_down_up = is_mouse_down.clone();
                        commands
                            .entity(*child)
                            .observe(move |_trigger: Trigger<Pointer<Down>>| {
                                *is_mouse_down.lock().unwrap() = true;
                            })
                            .observe(move |_trigger: Trigger<Pointer<Out>>| {
                                *is_mouse_down_out.lock().unwrap() = false;
                            })
                            .observe(move |_trigger: Trigger<Pointer<Up>>| {
                                if mem::take(&mut *is_mouse_down_up.lock().unwrap()) {
                                    on_click()
                                }
                            });
                    }
                }
            },
        );

        let pos = use_mut(&cx, || Vec2::ZERO);

        use_world(
            &cx,
            move |query: Query<(&Camera, &GlobalTransform)>,
                  q: Query<&GlobalTransform, Without<Camera>>,
                  ui_scale: Res<UiScale>| {
                let world_position = q.get(entity).unwrap().translation();

                let (camera, camera_transform) = query.single();
                let screen_pos = camera
                    .world_to_viewport(camera_transform, world_position)
                    .unwrap_or_default();
                SignalMut::set(pos, screen_pos / **ui_scale);
            },
        );

        let last_left_arm_rotation = use_ref(&cx, || Cell::new(0.));
        let last_right_arm_rotation = use_ref(&cx, || Cell::new(0.));

        let last_left_leg_rotation = use_ref(&cx, || Cell::new(0.));
        let last_right_leg_rotation = use_ref(&cx, || Cell::new(0.));

        use_world(
            &cx,
            move |models_query: Query<&VoxelSceneModels>,
                  mut transform_query: Query<&mut Transform>| {
                let Ok(models) = models_query.get(entity) else {
                    return;
                };

                for (name, rotation, last_rotation, offset) in [
                    (
                        "left_arm",
                        cx.me().left_arm_rotation,
                        last_left_arm_rotation,
                        Vec3::new(0., 24., 4.),
                    ),
                    (
                        "right_arm",
                        cx.me().right_arm_rotation,
                        last_right_arm_rotation,
                        Vec3::new(0., 24., 4.),
                    ),
                    (
                        "left_leg",
                        cx.me().left_leg_rotation,
                        last_left_leg_rotation,
                        Vec3::new(0., 3., 5.),
                    ),
                    (
                        "right_leg",
                        cx.me().right_leg_rotation,
                        last_right_leg_rotation,
                        Vec3::new(0., 3., 5.),
                    ),
                ] {
                    let entity = models.entities.get(name).unwrap();
                    let mut transform = transform_query.get_mut(*entity).unwrap();
                    transform.rotate_around(
                        offset,
                        Quat::from_rotation_x(rotation - last_rotation.get()),
                    );
                    last_rotation.set(rotation);
                }
            },
        );

        StatusBar {
            health: cx.me().health,
            energy: cx.me().energy,
            pos: *pos,
            is_selected: cx.me().is_selected,
        }
    }
}

#[derive(Data)]
struct StatusBar {
    health: u32,
    energy: u32,
    pos: Vec2,
    is_selected: bool,
}

impl Compose for StatusBar {
    fn compose(cx: Scope<Self>) -> impl Compose {
        let font = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("C&C Red Alert [INET].ttf")
        });
        let heart = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("heart.png")
        });
        let energy = use_world_once(&cx, |asset_server: Res<AssetServer>| {
            asset_server.load("energy.png")
        });

        let health_entity = use_bundle(&cx, || ());

        let entity = *use_context::<Entity>(&cx).unwrap();
        use_world_once(&cx, |mut commands: Commands| {
            commands.entity(entity).add_child(health_entity);
        });

        spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(cx.me().pos.y),
                left: Val::Px(cx.me().pos.x),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(0.5), Val::Px(0.25)),
                border: UiRect::all(Val::Px(0.25)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            BorderColor(if cx.me().is_selected {
                Color::WHITE
            } else {
                Color::BLACK
            }),
        ))
        .target(health_entity)
        .content((
            spawn((
                Node {
                    width: Val::Px(1.),
                    height: Val::Px(1.),
                    margin: UiRect::right(Val::Px(0.25)),
                    ..default()
                },
                UiImage::new(heart.clone()),
            )),
            spawn((
                Text::new(cx.me().health.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 2.,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )),
            spawn((
                Node {
                    width: Val::Px(1.),
                    height: Val::Px(1.),
                    margin: UiRect::new(Val::Px(0.5), Val::Px(0.25), Val::Px(0.), Val::Px(0.)),
                    ..default()
                },
                UiImage::new(energy.clone()),
            )),
            spawn((
                Text::new(cx.me().energy.to_string()),
                TextFont {
                    font: font.clone(),
                    font_size: 2.,
                    ..default()
                },
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            )),
        ))
    }
}
