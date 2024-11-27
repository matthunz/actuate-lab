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
    }
}
