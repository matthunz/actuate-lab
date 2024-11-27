use actuate::prelude::*;
use bevy::prelude::*;
use std::cell::Cell;
use voxy::{scene::VoxelSceneHandle, VoxelSceneModels};

#[derive(Data, Default)]
pub struct Character {
    pub transform: Transform,
    pub left_arm_rotation: f32,
    pub right_arm_rotation: f32,
    pub left_leg_rotation: f32,
    pub right_leg_rotation: f32,
    pub health: u32,
    pub energy: u32,
}

impl Compose for Character {
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
        }
    }
}

#[derive(Data)]
struct StatusBar {
    health: u32,
    energy: u32,
    pos: Vec2,
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

        spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(cx.me().pos.y),
                left: Val::Px(cx.me().pos.x),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(0.5), Val::Px(0.25)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
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
