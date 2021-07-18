use super::{
    super::{vek::*, Animation},
    QuadrupedMediumSkeleton, SkeletonAttr,
};

pub struct JumpAnimation;

impl Animation for JumpAnimation {
    type Dependency<'a> = f32;
    type Skeleton = QuadrupedMediumSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_medium_jump\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "quadruped_medium_jump")]
    fn update_skeleton_inner<'a>(
        skeleton: &Self::Skeleton,
        _global_time: Self::Dependency<'a>,
        _anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        next.neck.scale = Vec3::one() * 1.02;
        next.jaw.scale = Vec3::one() * 1.02;
        next.leg_fl.scale = Vec3::one() * 1.02;
        next.leg_fr.scale = Vec3::one() * 1.02;
        next.leg_bl.scale = Vec3::one() * 1.02;
        next.leg_br.scale = Vec3::one() * 1.02;
        next.foot_fl.scale = Vec3::one() * 0.96;
        next.foot_fr.scale = Vec3::one() * 0.96;
        next.foot_bl.scale = Vec3::one() * 0.96;
        next.foot_br.scale = Vec3::one() * 0.96;
        next.ears.scale = Vec3::one() * 1.02;

        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);

        next.neck.position = Vec3::new(0.0, s_a.neck.0, s_a.neck.1);

        next.jaw.position = Vec3::new(0.0, s_a.jaw.0, s_a.jaw.1);
        next.jaw.orientation = Quaternion::rotation_x(0.0);

        next.tail.position = Vec3::new(0.0, s_a.tail.0, s_a.tail.1);

        next.torso_front.position =
            Vec3::new(0.0, s_a.torso_front.0, s_a.torso_front.1) * s_a.scaler / 11.0;
        next.torso_front.orientation = Quaternion::rotation_y(0.0);

        next.torso_back.position = Vec3::new(0.0, s_a.torso_back.0, s_a.torso_back.1);

        next.ears.position = Vec3::new(0.0, s_a.ears.0, s_a.ears.1);

        next.leg_fl.position = Vec3::new(-s_a.leg_f.0, s_a.leg_f.1, s_a.leg_f.2);

        next.leg_fr.position = Vec3::new(s_a.leg_f.0, s_a.leg_f.1, s_a.leg_f.2);

        next.leg_bl.position = Vec3::new(-s_a.leg_b.0, s_a.leg_b.1, s_a.leg_b.2);

        next.leg_br.position = Vec3::new(s_a.leg_b.0, s_a.leg_b.1, s_a.leg_b.2);

        next.foot_fl.position = Vec3::new(-s_a.feet_f.0, s_a.feet_f.1, s_a.feet_f.2);

        next.foot_fr.position = Vec3::new(s_a.feet_f.0, s_a.feet_f.1, s_a.feet_f.2);

        next.foot_bl.position = Vec3::new(-s_a.feet_b.0, s_a.feet_b.1, s_a.feet_b.2);

        next.foot_br.position = Vec3::new(s_a.feet_b.0, s_a.feet_b.1, s_a.feet_b.2);

        next
    }
}
