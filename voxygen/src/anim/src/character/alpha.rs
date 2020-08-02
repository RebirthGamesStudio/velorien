use super::{super::{Animation, AnimationEventItem}, CharacterSkeleton, SkeletonAttr};
use common::comp::item::{Hands, ToolKind};
use std::f32::consts::PI;
use vek::*;
use std::collections::VecDeque;

pub struct AlphaAnimation;

impl Animation for AlphaAnimation {
    type Dependency = (Option<ToolKind>, Option<ToolKind>, f32, f64);
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_alpha\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "character_alpha")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, second_tool_kind, velocity, _global_time): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> (Self::Skeleton, VecDeque<AnimationEventItem>) {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let lab = 1.0;

        let foot = (((1.0)
            / (0.2
                + 0.8
                    * ((anim_time as f32 * lab as f32 * 2.0 * velocity).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 2.0 * velocity).sin());
        let slowersmooth = (anim_time as f32 * lab as f32 * 4.0).sin();
        let accel_med = 1.0 - (anim_time as f32 * 16.0 * lab as f32).cos();
        let accel_slow = 1.0 - (anim_time as f32 * 12.0 * lab as f32).cos();
        let accel_fast = 1.0 - (anim_time as f32 * 24.0 * lab as f32).cos();
        let decel = (anim_time as f32 * 16.0 * lab as f32).min(PI / 2.0).sin();
        let push = anim_time as f32 * lab as f32 * 4.0;
        let slow = (((5.0)
            / (0.4 + 4.6 * ((anim_time as f32 * lab as f32 * 9.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 9.0).sin());
        let quick = (((5.0)
            / (0.4 + 4.6 * ((anim_time as f32 * lab as f32 * 18.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 18.0).sin());
        let slower = (((1.0)
            / (0.0001 + 0.999 * ((anim_time as f32 * lab as f32 * 4.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 4.0).sin());
        let slowax = (((5.0)
            / (0.1 + 4.9 * ((anim_time as f32 * lab as f32 * 4.0 + 1.9).cos()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 4.0 + 1.9).cos());

        match active_tool_kind {
            //TODO: Inventory
            Some(ToolKind::Sword(_)) => {
                next.head.offset =
                    Vec3::new(0.0, -2.0 + skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.ori = Quaternion::rotation_z(slow * -0.25)
                    * Quaternion::rotation_x(0.0 + slow * 0.15)
                    * Quaternion::rotation_y(slow * -0.15);
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, skeleton_attr.chest.0, skeleton_attr.chest.1);
                next.chest.ori = Quaternion::rotation_z(slow * 0.4)
                    * Quaternion::rotation_x(0.0 + slow * -0.2)
                    * Quaternion::rotation_y(slow * 0.2);
                next.chest.scale = Vec3::one();

                next.belt.offset = Vec3::new(0.0, skeleton_attr.belt.0, skeleton_attr.belt.1);
                next.belt.ori = next.chest.ori * -0.3;

                next.shorts.offset = Vec3::new(0.0, skeleton_attr.shorts.0, skeleton_attr.shorts.1);
                next.shorts.ori = next.chest.ori * -0.45;

                next.l_hand.offset = Vec3::new(-0.75, -1.0, -2.5);
                next.l_hand.ori = Quaternion::rotation_x(1.27);
                next.l_hand.scale = Vec3::one() * 1.05;
                next.r_hand.offset = Vec3::new(0.75, -1.5, -5.5);
                next.r_hand.ori = Quaternion::rotation_x(1.27);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(0.0, 0.0, 0.0);
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);

                next.control.offset = Vec3::new(-10.0 + push * 5.0, 6.0 + push * 5.0, 2.0);
                next.control.ori = Quaternion::rotation_x(-1.4 + slow * 0.4)
                    * Quaternion::rotation_y(slow * -1.3)
                    * Quaternion::rotation_z(1.4 + slow * -0.5);
                next.control.scale = Vec3::one();

                next.l_foot.offset = Vec3::new(
                    -skeleton_attr.foot.0,
                    slow * -3.0 + quick * 3.0 - 4.0,
                    skeleton_attr.foot.2,
                );
                next.l_foot.ori = Quaternion::rotation_x(slow * 0.6)
                    * Quaternion::rotation_y((slow * -0.2).max(0.0));
                next.l_foot.scale = Vec3::one();

                next.r_foot.offset = Vec3::new(
                    skeleton_attr.foot.0,
                    slow * 3.0 + quick * -3.0 + 5.0,
                    skeleton_attr.foot.2,
                );
                next.r_foot.ori = Quaternion::rotation_x(slow * -0.6)
                    * Quaternion::rotation_y((slow * 0.2).min(0.0));
                next.r_foot.scale = Vec3::one();

                next.lantern.ori =
                    Quaternion::rotation_x(slow * -0.7 + 0.4) * Quaternion::rotation_y(slow * 0.4);

                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            Some(ToolKind::Dagger(_)) => {
                next.head.offset =
                    Vec3::new(0.0, -2.0 + skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.ori = Quaternion::rotation_z(slow * -0.25)
                    * Quaternion::rotation_x(0.0 + slow * 0.15)
                    * Quaternion::rotation_y(slow * -0.15);
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, skeleton_attr.chest.0, skeleton_attr.chest.1);
                next.chest.ori = Quaternion::rotation_z(slow * 0.4)
                    * Quaternion::rotation_x(0.0 + slow * -0.2)
                    * Quaternion::rotation_y(slow * 0.2);
                next.chest.scale = Vec3::one();

                next.belt.offset = Vec3::new(0.0, skeleton_attr.belt.0, skeleton_attr.belt.1);
                next.belt.ori = next.chest.ori * -0.3;

                next.shorts.offset = Vec3::new(0.0, skeleton_attr.shorts.0, skeleton_attr.shorts.1);
                next.shorts.ori = next.chest.ori * -0.45;

                // TODO: Fix animation
                next.l_hand.offset = Vec3::new(0.0, 0.0, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(0.0);
                next.l_hand.scale = Vec3::one() * 1.12;

                next.main.offset = Vec3::new(0.0, 0.0, 0.0);
                next.main.ori = Quaternion::rotation_x(0.0);

                next.l_control.offset = Vec3::new(-10.0 + push * 5.0, 6.0 + push * 5.0, 2.0);
                next.l_control.ori = Quaternion::rotation_x(-1.4 + slow * 0.4)
                    * Quaternion::rotation_y(slow * -1.3)
                    * Quaternion::rotation_z(1.4 + slow * -0.5);
                next.l_control.scale = Vec3::one();

                next.r_hand.offset = Vec3::new(0.0, 0.0, 0.0);
                next.r_hand.ori = Quaternion::rotation_x(0.0);
                next.r_hand.scale = Vec3::one() * 1.12;

                next.second.offset = Vec3::new(0.0, 0.0, 0.0);
                next.second.ori = Quaternion::rotation_x(0.0);

                next.r_control.offset = Vec3::new(8.0, 0.0, 0.0);
                next.r_control.ori = Quaternion::rotation_x(0.0);
                next.r_control.scale = Vec3::one();

                // next.r_control.offset = Vec3::new(-10.0 + push * 5.0, 6.0 + push * 5.0, 2.0);
                // next.r_control.ori = Quaternion::rotation_x(-1.4 + slow * 0.4)
                //     * Quaternion::rotation_y(slow * -1.3)
                //     * Quaternion::rotation_z(1.4 + slow * -0.5);
                // next.r_control.scale = Vec3::one();

                // next.r_hand.offset = Vec3::new(0.75, -1.5, -5.5);
                // next.r_hand.ori = Quaternion::rotation_x(1.27);
                // next.r_hand.scale = Vec3::one() * 1.05;

                // next.control.offset = Vec3::new(-10.0 + push * 5.0, 6.0 + push * 5.0, 2.0);
                // next.control.ori = Quaternion::rotation_x(-1.4 + slow * 0.4)
                //     * Quaternion::rotation_y(slow * -1.3)
                //     * Quaternion::rotation_z(1.4 + slow * -0.5);
                // next.control.scale = Vec3::one();

                next.l_foot.offset = Vec3::new(
                    -skeleton_attr.foot.0,
                    slow * -3.0 + quick * 3.0 - 4.0,
                    skeleton_attr.foot.2,
                );
                next.l_foot.ori = Quaternion::rotation_x(slow * 0.6)
                    * Quaternion::rotation_y((slow * -0.2).max(0.0));
                next.l_foot.scale = Vec3::one();

                next.r_foot.offset = Vec3::new(
                    skeleton_attr.foot.0,
                    slow * 3.0 + quick * -3.0 + 5.0,
                    skeleton_attr.foot.2,
                );
                next.r_foot.ori = Quaternion::rotation_x(slow * -0.6)
                    * Quaternion::rotation_y((slow * 0.2).min(0.0));
                next.r_foot.scale = Vec3::one();

                next.lantern.ori =
                    Quaternion::rotation_x(slow * -0.7 + 0.4) * Quaternion::rotation_y(slow * 0.4);

                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            Some(ToolKind::Axe(_)) => {
                next.head.offset = Vec3::new(
                    0.0 + slowax * 2.0,
                    0.0 + skeleton_attr.head.0 + slowax * -2.0,
                    skeleton_attr.head.1,
                );
                next.head.ori = Quaternion::rotation_z(slowax * 0.25)
                    * Quaternion::rotation_x(0.0 + slowax * 0.2)
                    * Quaternion::rotation_y(slowax * 0.2);
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, 0.0, 7.0);
                next.chest.ori = Quaternion::rotation_z(slowax * 0.2)
                    * Quaternion::rotation_x(0.0 + slowax * 0.2)
                    * Quaternion::rotation_y(slowax * 0.2);
                next.chest.scale = Vec3::one();

                next.belt.offset = Vec3::new(0.0, 0.0, -2.0);
                next.belt.ori = next.chest.ori * -0.2;

                next.shorts.offset = Vec3::new(0.0, 0.0, -5.0);
                next.shorts.ori = next.chest.ori * -0.15;

                next.l_hand.offset = Vec3::new(-4.0, 3.0, 2.0);
                next.l_hand.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_z(3.14 - 0.3)
                    * Quaternion::rotation_y(-0.8);
                next.l_hand.scale = Vec3::one() * 1.08;
                next.r_hand.offset = Vec3::new(-2.5, 9.0, 0.0);
                next.r_hand.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_z(3.14 - 0.3)
                    * Quaternion::rotation_y(-0.8);
                next.r_hand.scale = Vec3::one() * 1.06;
                next.main.offset = Vec3::new(-6.0, 10.0, -5.0);
                next.main.ori = Quaternion::rotation_x(1.27)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(-0.8);

                next.lantern.ori = Quaternion::rotation_x(slowax * -0.7 + 0.4)
                    * Quaternion::rotation_y(slowax * 0.4);

                next.control.offset = Vec3::new(0.0, 0.0 + slowax * 8.2, 6.0);
                next.control.ori = Quaternion::rotation_x(0.8)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(-0.7 + slowax * -1.9);
                next.control.scale = Vec3::one();
                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            Some(ToolKind::Hammer(_)) => {
                next.l_hand.offset = Vec3::new(-12.0, 0.0, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(-0.0) * Quaternion::rotation_y(0.0);
                next.l_hand.scale = Vec3::one() * 1.08;
                next.r_hand.offset = Vec3::new(3.0, 0.0, 0.0);
                next.r_hand.ori = Quaternion::rotation_x(0.0) * Quaternion::rotation_y(0.0);
                next.r_hand.scale = Vec3::one() * 1.06;
                next.main.offset = Vec3::new(0.0, 0.0, 0.0);
                next.main.ori = Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(-1.57)
                    * Quaternion::rotation_z(1.57);

                next.head.offset =
                    Vec3::new(0.0, -2.0 + skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.ori = Quaternion::rotation_z(slower * 0.03)
                    * Quaternion::rotation_x(slowersmooth * 0.1)
                    * Quaternion::rotation_y(slower * 0.05 + slowersmooth * 0.06)
                    * Quaternion::rotation_z((slowersmooth * -0.4).max(0.0));
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, 0.0, 7.0);
                next.chest.ori = Quaternion::rotation_z(slower * 0.18 + slowersmooth * 0.15)
                    * Quaternion::rotation_x(0.0 + slower * 0.18 + slowersmooth * 0.15)
                    * Quaternion::rotation_y(slower * 0.18 + slowersmooth * 0.15);

                next.belt.offset = Vec3::new(0.0, 0.0, -2.0);
                next.belt.ori = Quaternion::rotation_z(slower * -0.1 + slowersmooth * -0.075)
                    * Quaternion::rotation_x(0.0 + slower * -0.1)
                    * Quaternion::rotation_y(slower * -0.1);

                next.shorts.offset = Vec3::new(0.0, 0.0, -5.0);
                next.shorts.ori = Quaternion::rotation_z(slower * -0.1 + slowersmooth * -0.075)
                    * Quaternion::rotation_x(0.0 + slower * -0.1)
                    * Quaternion::rotation_y(slower * -0.1);

                next.lantern.ori = Quaternion::rotation_x(slower * -0.7 + 0.4)
                    * Quaternion::rotation_y(slower * 0.4);

                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;

                if velocity > 0.5 {
                    next.l_foot.offset =
                        Vec3::new(-skeleton_attr.foot.0, foot * -6.0, skeleton_attr.foot.2);
                    next.l_foot.ori = Quaternion::rotation_x(foot * -0.4)
                        * Quaternion::rotation_z((slower * 0.3).max(0.0));
                    next.l_foot.scale = Vec3::one();

                    next.r_foot.offset =
                        Vec3::new(skeleton_attr.foot.0, foot * 6.0, skeleton_attr.foot.2);
                    next.r_foot.ori = Quaternion::rotation_x(foot * 0.4)
                        * Quaternion::rotation_z((slower * 0.3).max(0.0));
                    next.r_foot.scale = Vec3::one();
                    next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                    next.torso.ori = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(-0.15);
                    next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
                } else {
                    next.l_foot.offset = Vec3::new(
                        -skeleton_attr.foot.0,
                        -2.5,
                        skeleton_attr.foot.2 + (slower * 2.5).max(0.0),
                    );
                    next.l_foot.ori = Quaternion::rotation_x(slower * -0.2 - 0.2)
                        * Quaternion::rotation_z((slower * 1.0).max(0.0));
                    next.l_foot.scale = Vec3::one();

                    next.r_foot.offset = Vec3::new(
                        skeleton_attr.foot.0,
                        3.5 - slower * 2.0,
                        skeleton_attr.foot.2,
                    );
                    next.r_foot.ori = Quaternion::rotation_x(slower * 0.1)
                        * Quaternion::rotation_z((slower * 0.5).max(0.0));
                    next.r_foot.scale = Vec3::one();
                    next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                    next.torso.ori = Quaternion::rotation_z(0.0);
                    next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
                }

                //next.control.offset = Vec3::new(-4.0, 3.0 + slower * 2.0, 5.0 + slower *
                // 5.0); next.control.ori = Quaternion::rotation_x()
                //    * Quaternion::rotation_y(0.0)
                //    * Quaternion::rotation_z(1.4);
                next.control.scale = Vec3::one();
                next.control.offset = Vec3::new(-8.0, 7.0, 1.0);
                next.control.ori = Quaternion::rotation_x(-1.5 + slower * 1.5)
                    * Quaternion::rotation_y(slowersmooth * 0.35 - 0.3)
                    * Quaternion::rotation_z(1.4 + slowersmooth * 0.2);
                next.control.scale = Vec3::one();

                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_z(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            Some(ToolKind::Staff(_)) => {
                next.head.offset = Vec3::new(
                    0.0,
                    0.0 + skeleton_attr.head.0, /* + decel * 0.8 */
                    // Had some clipping issues
                    skeleton_attr.head.1,
                );
                next.head.ori = Quaternion::rotation_z(decel * 0.25)
                    * Quaternion::rotation_x(0.0 + decel * 0.1)
                    * Quaternion::rotation_y(decel * -0.1);

                next.chest.ori = Quaternion::rotation_z(decel * -0.2)
                    * Quaternion::rotation_x(0.0 + decel * -0.2)
                    * Quaternion::rotation_y(decel * 0.2);

                next.belt.ori = Quaternion::rotation_z(decel * -0.1)
                    * Quaternion::rotation_x(0.0 + decel * -0.1)
                    * Quaternion::rotation_y(decel * 0.1);

                next.shorts.offset = Vec3::new(0.0, 0.0, -5.0);
                next.shorts.ori = Quaternion::rotation_z(decel * -0.08)
                    * Quaternion::rotation_x(0.0 + decel * -0.08)
                    * Quaternion::rotation_y(decel * 0.08);
                next.l_hand.offset = Vec3::new(0.0, 1.0, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(1.27);
                next.l_hand.scale = Vec3::one() * 1.05;
                next.r_hand.offset = Vec3::new(0.0, 0.0, 10.0);
                next.r_hand.ori = Quaternion::rotation_x(1.27);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(0.0, 6.0, -4.0);
                next.main.ori = Quaternion::rotation_x(-0.3);

                next.control.offset = Vec3::new(-8.0 - slow * 1.0, 3.0 - slow * 5.0, 0.0);
                next.control.ori = Quaternion::rotation_x(-1.2)
                    * Quaternion::rotation_y(slow * 1.5)
                    * Quaternion::rotation_z(1.4 + slow * 0.5);
                next.control.scale = Vec3::one();
                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            Some(ToolKind::Shield(_)) => {
                next.head.offset = Vec3::new(
                    0.0,
                    0.0 + skeleton_attr.head.0 + decel * 0.8,
                    skeleton_attr.head.1,
                );
                next.head.ori = Quaternion::rotation_z(decel * 0.25)
                    * Quaternion::rotation_x(0.0 + decel * 0.1)
                    * Quaternion::rotation_y(decel * -0.1);
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, 0.0, 7.0);
                next.chest.ori = Quaternion::rotation_z(decel * -0.2)
                    * Quaternion::rotation_x(0.0 + decel * -0.2)
                    * Quaternion::rotation_y(decel * 0.2);

                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;

                next.belt.offset = Vec3::new(0.0, 0.0, 0.0);
                next.belt.ori = Quaternion::rotation_z(decel * -0.1)
                    * Quaternion::rotation_x(0.0 + decel * -0.1)
                    * Quaternion::rotation_y(decel * 0.1);

                next.shorts.offset = Vec3::new(0.0, 0.0, 0.0);
                next.belt.ori = Quaternion::rotation_z(decel * -0.08)
                    * Quaternion::rotation_x(0.0 + decel * -0.08)
                    * Quaternion::rotation_y(decel * 0.08);

                next.l_control.offset =
                    Vec3::new(-8.0 + accel_slow * 10.0, 8.0 + accel_fast * 3.0, 0.0);
                next.l_control.ori = Quaternion::rotation_z(-0.8)
                    * Quaternion::rotation_x(0.0 + accel_med * -0.8)
                    * Quaternion::rotation_y(0.0 + accel_med * -0.4);

                next.l_hand.offset = Vec3::new(0.0, 0.0, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(0.0);
                next.l_hand.scale = Vec3::one() * 1.01;

                next.main.offset = Vec3::new(0.0, 0.0, 0.0);
                next.main.ori = Quaternion::rotation_z(0.0);

                next.r_control.offset = Vec3::new(8.0, 0.0, 0.0);
                next.r_control.ori = Quaternion::rotation_x(0.0);

                next.r_hand.offset = Vec3::new(0.0, 0.0, 0.0);
                next.r_hand.ori = Quaternion::rotation_x(0.0);
                next.r_hand.scale = Vec3::one() * 1.01;

                next.second.offset = Vec3::new(0.0, 0.0, 0.0);
                next.second.ori = Quaternion::rotation_x(0.0);
            },
            Some(ToolKind::Debug(_)) => {
                next.head.offset = Vec3::new(
                    0.0,
                    -2.0 + skeleton_attr.head.0 + decel * 0.8,
                    skeleton_attr.head.1,
                );
                next.head.ori = Quaternion::rotation_x(0.0);
                next.head.scale = Vec3::one() * skeleton_attr.head_scale;

                next.chest.offset = Vec3::new(0.0, 0.0, 7.0);
                next.chest.ori = Quaternion::rotation_z(decel * -0.2)
                    * Quaternion::rotation_x(0.0 + decel * -0.2)
                    * Quaternion::rotation_y(decel * 0.2);

                next.l_hand.offset =
                    Vec3::new(-8.0 + accel_slow * 10.0, 8.0 + accel_fast * 3.0, 0.0);
                next.l_hand.ori = Quaternion::rotation_z(-0.8)
                    * Quaternion::rotation_x(accel_med * -0.8)
                    * Quaternion::rotation_y(accel_med * -0.4);
                next.l_hand.scale = Vec3::one() * 1.01;

                next.r_hand.offset =
                    Vec3::new(-8.0 + accel_slow * 10.0, 8.0 + accel_fast * 3.0, -2.0);
                next.r_hand.ori = Quaternion::rotation_z(-0.8)
                    * Quaternion::rotation_x(accel_med * -0.8)
                    * Quaternion::rotation_y(accel_med * -0.4);
                next.r_hand.scale = Vec3::one() * 1.01;

                next.main.offset = Vec3::new(-8.0 + accel_slow * 10.0, 8.0 + accel_fast * 3.0, 0.0);
                next.main.ori = Quaternion::rotation_z(-0.8)
                    * Quaternion::rotation_x(0.0 + accel_med * -0.8)
                    * Quaternion::rotation_y(0.0 + accel_med * -0.4);
                next.main.scale = Vec3::one();
                next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
                next.torso.ori = Quaternion::rotation_x(0.0);
                next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;
            },
            _ => {},
        }
        next.lantern.offset = Vec3::new(
            skeleton_attr.lantern.0,
            skeleton_attr.lantern.1,
            skeleton_attr.lantern.2,
        );
        next.lantern.scale = Vec3::one() * 0.65;
        next.l_shoulder.scale = Vec3::one() * 1.1;
        next.r_shoulder.scale = Vec3::one() * 1.1;
        next.glider.offset = Vec3::new(0.0, 0.0, 10.0);
        next.glider.scale = Vec3::one() * 0.0;
        next.l_control.scale = Vec3::one();
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
