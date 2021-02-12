use super::{
    super::{vek::*, Animation},
    GolemSkeleton, SkeletonAttr,
};
use common::states::utils::StageSection;
use std::f32::consts::PI;

pub struct AlphaAnimation;

impl Animation for AlphaAnimation {
    type Dependency = (Option<StageSection>, f64, f64);
    type Skeleton = GolemSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"golem_alpha\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "golem_alpha")]

    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (stage_section, global_time, timer): Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let (move1base, move2base, move3) = match stage_section {
            Some(StageSection::Buildup) => ((anim_time as f32).powf(0.25), 0.0, 0.0),
            Some(StageSection::Swing) => (1.0, anim_time as f32, 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, (anim_time as f32).powf(4.0)),
            _ => (0.0, 0.0, 0.0),
        };

        let pullback = 1.0 - move3;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum() as f32;

        let move1 = move1base * pullback;
        let move2 = move2base * pullback;
        if mirror > 0.0 {
            next.head.orientation =
                Quaternion::rotation_x(-0.2) * Quaternion::rotation_z(move1 * -1.2 + move2 * 2.0);

            next.upper_torso.orientation = Quaternion::rotation_x(move1 * -0.6)
                * Quaternion::rotation_z(move1 * 1.2 + move2 * -3.2);

            next.lower_torso.orientation = Quaternion::rotation_z(move1 * -1.2 + move2 * 3.2)
                * Quaternion::rotation_x(move1 * 0.6);

            next.shoulder_l.orientation = Quaternion::rotation_y(move1 * 0.8)
                * Quaternion::rotation_x(move1 * -1.0 + move2 * 1.6);

            next.shoulder_r.orientation = Quaternion::rotation_x(move1 * 0.4);

            next.hand_l.orientation =
                Quaternion::rotation_z(0.0) * Quaternion::rotation_x(move1 * -1.0 + move2 * 1.8);

            next.hand_r.orientation =
                Quaternion::rotation_y(move1 * 0.5) * Quaternion::rotation_x(move1 * 0.4);
        } else {
            next.head.orientation =
                Quaternion::rotation_x(-0.2) * Quaternion::rotation_z(move1 * 1.2 + move2 * -2.0);

            next.upper_torso.orientation = Quaternion::rotation_x(move1 * -0.6)
                * Quaternion::rotation_z(move1 * -1.2 + move2 * 3.2);

            next.lower_torso.orientation = Quaternion::rotation_z(move1 * 1.2 + move2 * -3.2)
                * Quaternion::rotation_x(move1 * 0.6);

            next.shoulder_l.orientation = Quaternion::rotation_x(move1 * 0.4);

            next.shoulder_r.orientation = Quaternion::rotation_y(move1 * -0.8)
                * Quaternion::rotation_x(move1 * -1.0 + move2 * 1.6);

            next.hand_l.orientation =
                Quaternion::rotation_y(move1 * -0.5) * Quaternion::rotation_x(move1 * 0.4);

            next.hand_r.orientation =
                Quaternion::rotation_y(0.0) * Quaternion::rotation_x(move1 * -1.0 + move2 * 1.8);
        };
        next.torso.position = Vec3::new(0.0, move1 * 0.7, move1 * -0.3);
        next
    }
}
