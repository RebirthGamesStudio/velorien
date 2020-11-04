use super::{super::Animation, SkeletonAttr, TheropodSkeleton};
//use std::f32::consts::PI;
use super::super::vek::*;

pub struct JumpAnimation;

impl Animation for JumpAnimation {
    type Dependency = (f32, f64);
    type Skeleton = TheropodSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"theropod_jump\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "theropod_jump")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        _global_time: Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let breathe = (anim_time as f32 * 0.8).sin();

        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1 + breathe * 0.3);
        next.head.orientation = Quaternion::rotation_x(breathe * 0.1 - 0.1);
        next.head.scale = Vec3::one() * 1.02;

        next.jaw.position = Vec3::new(0.0, s_a.jaw.0, s_a.jaw.1);
        next.jaw.orientation = Quaternion::rotation_x(breathe * 0.05);
        next.jaw.scale = Vec3::one() * 0.98;

        next.neck.position = Vec3::new(0.0, s_a.neck.0, s_a.neck.1 + breathe * 0.2);
        next.neck.orientation = Quaternion::rotation_x(-0.1);
        next.neck.scale = Vec3::one() * 0.98;

        next.chest_front.position =
            Vec3::new(0.0, s_a.chest_front.0, s_a.chest_front.1 + breathe * 0.3) / s_a.scaler;
        next.chest_front.orientation = Quaternion::rotation_x(breathe * 0.04);
        next.chest_front.scale = Vec3::one() / s_a.scaler;

        next.chest_back.position = Vec3::new(0.0, s_a.chest_back.0, s_a.chest_back.1);
        next.chest_back.orientation = Quaternion::rotation_x(breathe * -0.04);
        next.chest_back.scale = Vec3::one();

        next.tail_front.position = Vec3::new(0.0, s_a.tail_front.0, s_a.tail_front.1);
        next.tail_front.orientation = Quaternion::rotation_x(0.1);
        next.tail_front.scale = Vec3::one();

        next.tail_back.position = Vec3::new(0.0, s_a.tail_back.0, s_a.tail_back.1);
        next.tail_back.orientation = Quaternion::rotation_x(0.1);
        next.tail_back.scale = Vec3::one();

        next.hand_l.position = Vec3::new(-s_a.hand.0, s_a.hand.1, s_a.hand.2);
        next.hand_l.orientation = Quaternion::rotation_x(breathe * 0.2);
        next.hand_l.scale = Vec3::one();

        next.hand_r.position = Vec3::new(s_a.hand.0, s_a.hand.1, s_a.hand.2);
        next.hand_r.orientation = Quaternion::rotation_x(breathe * 0.2);
        next.hand_r.scale = Vec3::one();

        next.leg_l.position = Vec3::new(-s_a.leg.0, s_a.leg.1, s_a.leg.2 + breathe * 0.05);
        next.leg_l.orientation = Quaternion::rotation_z(0.0);
        next.leg_l.scale = Vec3::one();

        next.leg_r.position = Vec3::new(s_a.leg.0, s_a.leg.1, s_a.leg.2 + breathe * 0.05);
        next.leg_r.orientation = Quaternion::rotation_z(0.0);
        next.leg_r.scale = Vec3::one();

        next.foot_l.position = Vec3::new(-s_a.foot.0, s_a.foot.1, s_a.foot.2 + breathe * -0.35);
        next.foot_l.orientation = Quaternion::rotation_z(0.0);
        next.foot_l.scale = Vec3::one() * 1.02;

        next.foot_r.position = Vec3::new(s_a.foot.0, s_a.foot.1, s_a.foot.2 + breathe * -0.45);
        next.foot_r.orientation = Quaternion::rotation_z(0.0);
        next.foot_r.scale = Vec3::one() * 1.02;

        next
    }
}
