use super::{super::{Animation, AnimationEventItem}, CharacterSkeleton, SkeletonAttr};
use common::comp::item::{Hands, ToolKind};
use std::f32::consts::PI;
use vek::*;
use std::collections::VecDeque;

pub struct Input {
    pub attack: bool,
}
pub struct SpinAnimation;

impl Animation for SpinAnimation {
    type Dependency = (Option<ToolKind>, Option<ToolKind>, f64);
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_spin\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "character_spin")]
    #[allow(clippy::unnested_or_patterns)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, second_tool_kind, _global_time): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> (Self::Skeleton, VecDeque<AnimationEventItem>) {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let lab = 1.0;

        let foot = (((5.0)
            / (1.1 + 3.9 * ((anim_time as f32 * lab as f32 * 10.32).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 10.32).sin());

        let decel = (anim_time as f32 * 16.0 * lab as f32).min(PI / 2.0).sin();

        let spin = (anim_time as f32 * 2.8 * lab as f32).sin();
        let spinhalf = (anim_time as f32 * 1.4 * lab as f32).sin();

        match active_tool_kind {
            //TODO: Inventory
            Some(ToolKind::Axe(_))
            | Some(ToolKind::Hammer(_))
            | Some(ToolKind::Sword(_))
            | Some(ToolKind::Dagger(_)) => {
                //INTENTION: SWORD
                next.l_hand.offset = Vec3::new(-0.75, -1.0, -2.5);
                next.l_hand.ori = Quaternion::rotation_x(1.27);
                next.l_hand.scale = Vec3::one() * 1.04;
                next.r_hand.offset = Vec3::new(0.75, -1.5, -5.5);
                next.r_hand.ori = Quaternion::rotation_x(1.27);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(0.0, 6.0, -1.0);
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(-4.5 + spinhalf * 4.0, 11.0, 8.0);
                next.control.ori = Quaternion::rotation_x(-1.7)
                    * Quaternion::rotation_y(0.2 + spin * -2.0)
                    * Quaternion::rotation_z(1.4 + spin * 0.1);
                next.control.scale = Vec3::one();
                next.head.offset = Vec3::new(
                    0.0,
                    -2.0 + skeleton_attr.head.0 + spin * -0.8,
                    skeleton_attr.head.1,
                );
                next.head.ori = Quaternion::rotation_z(spin * -0.25)
                    * Quaternion::rotation_x(0.0 + spin * -0.1)
                    * Quaternion::rotation_y(spin * -0.2);
                next.chest.offset = Vec3::new(0.0, skeleton_attr.chest.0, skeleton_attr.chest.1);
                next.chest.ori = Quaternion::rotation_z(spin * 0.1)
                    * Quaternion::rotation_x(0.0 + spin * 0.1)
                    * Quaternion::rotation_y(decel * -0.2);
                next.chest.scale = Vec3::one();

                next.belt.offset = Vec3::new(0.0, 0.0, -2.0);
                next.belt.ori = next.chest.ori * -0.1;
                next.belt.scale = Vec3::one();

                next.shorts.offset = Vec3::new(0.0, 0.0, -5.0);
                next.belt.ori = next.chest.ori * -0.08;
                next.shorts.scale = Vec3::one();
                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z((spin * 7.0).max(0.3))
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },

            _ => {},
        }
        next.l_foot.offset = Vec3::new(-skeleton_attr.foot.0, foot * 1.0, skeleton_attr.foot.2);
        next.l_foot.ori = Quaternion::rotation_x(foot * -1.2);
        next.l_foot.scale = Vec3::one();

        next.r_foot.offset = Vec3::new(skeleton_attr.foot.0, foot * -1.0, skeleton_attr.foot.2);
        next.r_foot.ori = Quaternion::rotation_x(foot * 1.2);
        next.r_foot.scale = Vec3::one();

        next.l_shoulder.offset = Vec3::new(-5.0, 0.0, 4.7);
        next.l_shoulder.ori = Quaternion::rotation_x(0.0);
        next.l_shoulder.scale = Vec3::one() * 1.1;

        next.r_shoulder.offset = Vec3::new(5.0, 0.0, 4.7);
        next.r_shoulder.ori = Quaternion::rotation_x(0.0);
        next.r_shoulder.scale = Vec3::one() * 1.1;

        next.glider.offset = Vec3::new(0.0, 5.0, 0.0);
        next.glider.ori = Quaternion::rotation_y(0.0);
        next.glider.scale = Vec3::one() * 0.0;

        next.lantern.offset = Vec3::new(
            skeleton_attr.lantern.0,
            skeleton_attr.lantern.1,
            skeleton_attr.lantern.2,
        );
        next.lantern.ori =
            Quaternion::rotation_x(spin * -0.7 + 0.4) * Quaternion::rotation_y(spin * 0.4);
        next.lantern.scale = Vec3::one() * 0.65;

        next.l_control.offset = Vec3::new(0.0, 0.0, 0.0);
        next.l_control.ori = Quaternion::rotation_x(0.0);
        next.l_control.scale = Vec3::one();

        next.r_control.offset = Vec3::new(0.0, 0.0, 0.0);
        next.r_control.ori = Quaternion::rotation_x(0.0);
        next.r_control.scale = Vec3::one();

        next.second.scale = match (
            active_tool_kind.map(|tk| tk.into_hands()),
            second_tool_kind.map(|tk| tk.into_hands()),
        ) {
            (Some(Hands::OneHand), Some(Hands::OneHand)) => Vec3::one(),
            (_, _) => Vec3::zero(),
        };

        (next, VecDeque::new())
    }
}
