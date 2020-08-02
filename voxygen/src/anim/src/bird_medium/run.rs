use super::{super::{Animation, AnimationEventItem}, BirdMediumSkeleton, SkeletonAttr};
use std::f32::consts::PI;
use vek::*;
use std::collections::VecDeque;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Dependency = (f32, f64);
    type Skeleton = BirdMediumSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"bird_medium_run\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "bird_medium_run")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, _global_time): Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> (Self::Skeleton, VecDeque<AnimationEventItem>) {
        let mut next = (*skeleton).clone();

        let lab = 16.0; //14.0

        let footl = (anim_time as f32 * lab as f32 + PI).sin();
        let footr = (anim_time as f32 * lab as f32).sin();
        let center = (anim_time as f32 * lab as f32 + PI / 2.0).sin();
        let centeroffset = (anim_time as f32 * lab as f32 + PI * 1.5).sin();

        next.head.offset = Vec3::new(
            0.0,
            skeleton_attr.head.0,
            skeleton_attr.head.1 + center * 0.5,
        );
        next.head.ori = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0 + center * 0.03);
        next.head.scale = Vec3::one();

        next.torso.offset = Vec3::new(
            0.0,
            skeleton_attr.chest.0 + centeroffset * 0.6,
            center * 0.6 + skeleton_attr.chest.1,
        ) / 11.0;
        next.torso.ori = Quaternion::rotation_y(center * 0.05);
        next.torso.scale = Vec3::one() / 11.0;

        next.tail.offset = Vec3::new(
            0.0,
            skeleton_attr.tail.0,
            skeleton_attr.tail.1 + centeroffset * 0.6,
        );
        next.tail.ori = Quaternion::rotation_x(center * 0.03);
        next.tail.scale = Vec3::one();

        next.wing_l.offset = Vec3::new(
            -skeleton_attr.wing.0,
            skeleton_attr.wing.1,
            skeleton_attr.wing.2,
        );
        next.wing_l.ori = Quaternion::rotation_y((footl * 0.35).max(0.0));
        next.wing_l.scale = Vec3::one() * 1.05;

        next.wing_r.offset = Vec3::new(
            skeleton_attr.wing.0,
            skeleton_attr.wing.1,
            skeleton_attr.wing.2,
        );
        next.wing_r.ori = Quaternion::rotation_y((footr * 0.35).min(0.0));
        next.wing_r.scale = Vec3::one() * 1.05;

        next.leg_l.offset = Vec3::new(
            -skeleton_attr.foot.0,
            skeleton_attr.foot.1 + footl * 1.0,
            skeleton_attr.foot.2,
        ) / 11.0;
        next.leg_l.ori = Quaternion::rotation_x(footl * 0.5);
        next.leg_l.scale = Vec3::one() / 11.0;

        next.leg_r.offset = Vec3::new(
            skeleton_attr.foot.0,
            skeleton_attr.foot.1 + footr * 1.0,
            skeleton_attr.foot.2,
        ) / 11.0;
        next.leg_r.ori = Quaternion::rotation_x(footr * 0.5);
        next.leg_r.scale = Vec3::one() / 11.0;
        (next, VecDeque::new())
    }
}
