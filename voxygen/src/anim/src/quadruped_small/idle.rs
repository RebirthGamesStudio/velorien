use super::{super::{Animation, AnimationEventItem}, QuadrupedSmallSkeleton, SkeletonAttr};
use std::{f32::consts::PI, ops::Mul};
use vek::*;
use std::collections::VecDeque;

pub struct IdleAnimation;

impl Animation for IdleAnimation {
    type Dependency = f64;
    type Skeleton = QuadrupedSmallSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_small_idle\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "quadruped_small_idle")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        global_time: Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> (Self::Skeleton, VecDeque<AnimationEventItem>) {
        let mut next = (*skeleton).clone();

        let slow = (anim_time as f32 * 3.5).sin();
        let slowa = (anim_time as f32 * 3.5 + PI / 2.0).sin();

        let slow_alt = (anim_time as f32 * 3.5 + PI).sin();

        let head_look = Vec2::new(
            ((global_time + anim_time) as f32 / 8.0)
                .floor()
                .mul(7331.0)
                .sin()
                * 0.5,
            ((global_time + anim_time) as f32 / 8.0)
                .floor()
                .mul(1337.0)
                .sin()
                * 0.25,
        );

        next.head.offset =
            Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1 + slow * 0.2) / 11.0;
        next.head.ori = Quaternion::rotation_z(head_look.x)
            * Quaternion::rotation_x(head_look.y + slow_alt * 0.03);
        next.head.scale = Vec3::one() / 10.5;

        next.chest.offset = Vec3::new(
            slow * 0.05,
            skeleton_attr.chest.0,
            skeleton_attr.chest.1 + slowa * 0.2,
        ) / 11.0;
        next.chest.ori = Quaternion::rotation_y(slow * 0.05);
        next.chest.scale = Vec3::one() / 11.0;

        next.leg_fl.offset = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        ) / 11.0;
        next.leg_fl.ori = Quaternion::rotation_x(slow * 0.08);
        next.leg_fl.scale = Vec3::one() / 11.0;

        next.leg_fr.offset = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        ) / 11.0;
        next.leg_fr.ori = Quaternion::rotation_x(slow_alt * 0.08);
        next.leg_fr.scale = Vec3::one() / 11.0;

        next.leg_bl.offset = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        ) / 11.0;
        next.leg_bl.ori = Quaternion::rotation_x(slow_alt * 0.08);
        next.leg_bl.scale = Vec3::one() / 11.0;

        next.leg_br.offset = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        ) / 11.0;
        next.leg_br.ori = Quaternion::rotation_x(slow * 0.08);
        next.leg_br.scale = Vec3::one() / 11.0;

        next.tail.offset = Vec3::new(0.0, skeleton_attr.tail.0, skeleton_attr.tail.1);
        next.tail.ori = Quaternion::rotation_z(slow * 0.4);
        next.tail.scale = Vec3::one();

        (next, VecDeque::new())
    }
}
