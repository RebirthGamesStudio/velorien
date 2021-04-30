use super::{super::{AnimationEvent, Animation}, BirdSmallSkeleton, SkeletonAttr};
//use std::{f32::consts::PI, ops::Mul};
use super::super::vek::*;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Dependency = (f32, f32);
    type Skeleton = BirdSmallSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"bird_small_run\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "bird_small_run")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, _global_time): Self::Dependency,
        _anim_time: f32,
        _rate: &mut f32,
        _skeleton_attr: &SkeletonAttr,
    ) -> (Self::Skeleton, Vec<AnimationEvent>) {
        let mut next = (*skeleton).clone();
        let anim_events: Vec<AnimationEvent> = Vec::new();

        next.head.position = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.head.orientation = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.head.scale = Vec3::one() / 10.88;

        next.torso.position = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.torso.orientation = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.torso.scale = Vec3::one() / 10.88;

        next.wing_l.position = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.wing_l.orientation = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.wing_l.scale = Vec3::one() / 10.88;

        next.wing_r.position = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.wing_r.orientation = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.wing_r.scale = Vec3::one() / 10.88;

        (next, anim_events)
    }
}
