use super::{
    super::{vek::*, Animation},
    BipedLargeSkeleton, SkeletonAttr,
};
use common::{
    comp::item::{Hands, ToolKind},
    states::utils::StageSection,
};
use std::f32::consts::PI;

pub struct AlphaAnimation;

impl Animation for AlphaAnimation {
    type Dependency = (
        Option<ToolKind>,
        Option<ToolKind>,
        f32,
        f64,
        Option<StageSection>,
    );
    type Skeleton = BipedLargeSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"biped_large_alpha\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "biped_large_alpha")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, second_tool_kind, velocity, global_time, stage_section): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let lab = 1.0;

        let (movement1, movement2, movement3) = match stage_section {
            Some(StageSection::Buildup) => (anim_time as f32, 0.0, 0.0),
            Some(StageSection::Swing) => (1.0, anim_time as f32, 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, anim_time as f32),
            _ => (0.0, 0.0, 0.0),
        };

        let foot = (((1.0)
            / (0.2
                + 0.8
                    * ((anim_time as f32 * lab as f32 * 2.0 * velocity).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 2.0 * velocity).sin());
        let slowersmooth = (anim_time as f32 * lab as f32 * 4.0).sin();
        let decel = (anim_time as f32 * 16.0 * lab as f32).min(PI / 2.0).sin();
        let slow = (((5.0)
            / (0.4 + 4.6 * ((anim_time as f32 * lab as f32 * 9.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 9.0).sin());
        let quick = (((5.0)
            / (0.4 + 4.6 * ((anim_time as f32 * lab as f32 * 18.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 18.0).sin());
        let axe = (((1.0)
            / (0.05 + 0.95 * ((anim_time as f32 * lab as f32 * 8.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 8.0).sin());
        let staff = (((1.0)
            / (0.05 + 0.95 * ((anim_time as f32 * lab as f32 * 10.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 10.0).sin());

        let slower = (((1.0)
            / (0.05
                + 0.95
                    * ((anim_time as f32 * lab as f32 * 8.0 - 0.5 * PI).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 8.0 - 0.5 * PI).sin())
            + 1.0;
        let twist = (anim_time as f32 * lab as f32 * 4.0).sin() + 0.5;

        let random = ((((2.0
            * (((global_time as f32 - anim_time as f32) * 10.0)
                - (((global_time as f32 - anim_time as f32) * 10.0).round())))
        .abs())
            * 10.0)
            .round())
            / 10.0;

        let switch = if random > 0.5 { 1.0 } else { -1.0 };

        match active_tool_kind {
            Some(ToolKind::Sword(_)) => {
                next.hand_l.position = Vec3::new(-4.75, -1.0, 2.5);
                next.hand_l.orientation = Quaternion::rotation_x(1.47) * Quaternion::rotation_y(-0.2);
                next.hand_l.scale = Vec3::one() * 1.04;
                next.hand_r.position = Vec3::new(0.75, -1.5, -0.5);
                next.hand_r.orientation = Quaternion::rotation_x(1.47) * Quaternion::rotation_y(0.3);
                next.hand_r.scale = Vec3::one() * 1.05;
                next.main.position = Vec3::new(0.0, 5.0, -6.0);
                next.main.orientation = Quaternion::rotation_x(-0.1)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);

                next.head.position = Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1);

                next.control.position = Vec3::new(
                    -7.0,
                    7.0 + movement1 * -4.0 + movement2 * 16.0 + movement3 * -4.0,
                    2.0 + movement1 * 1.0,
                );
                next.control.orientation = Quaternion::rotation_x(movement1 * -0.5)
                    * Quaternion::rotation_y(movement1 * -1.0 + movement2 * -0.6 + movement3 * 1.0)
                    * Quaternion::rotation_z(movement1 * -1.2 + movement2 * 1.3);

                next.upper_torso.orientation = Quaternion::rotation_z(
                    movement1 * 1.5 + (movement2 * 1.75).sin() * -3.0 + movement3 * 0.5,
                );

                next.head.position = Vec3::new(0.0, skeleton_attr.head.0 + 0.0, skeleton_attr.head.1);
                next.head.orientation = Quaternion::rotation_z(
                    movement1 * -0.9 + (movement2 * 1.75).sin() * 2.5 + movement3 * -0.5,
                );
            },
            Some(ToolKind::Dagger(_)) => {
                next.head.position =
                    Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.orientation = Quaternion::rotation_z(slow * -0.25)
                    * Quaternion::rotation_x(0.0 + slow * 0.15)
                    * Quaternion::rotation_y(slow * -0.15);
                next.head.scale = Vec3::one() * 1.01;

                next.upper_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.upper_torso.0,
                    skeleton_attr.upper_torso.1,
                );
                next.upper_torso.orientation = Quaternion::rotation_z(slow * 0.4)
                    * Quaternion::rotation_x(0.0 + slow * -0.2)
                    * Quaternion::rotation_y(slow * 0.2);
                next.upper_torso.scale = Vec3::one();

                next.lower_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.lower_torso.0,
                    skeleton_attr.lower_torso.1,
                );
                next.lower_torso.orientation = next.upper_torso.orientation * -0.45;

                next.hand_l.position = Vec3::new(0.0, 0.0, 0.0);
                next.hand_l.orientation = Quaternion::rotation_x(0.0);
                next.hand_l.scale = Vec3::one() * 1.12;

                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_x(0.0);

                next.hand_r.position = Vec3::new(0.0, 0.0, 0.0);
                next.hand_r.orientation = Quaternion::rotation_x(0.0);
                next.hand_r.scale = Vec3::one() * 1.12;

                next.second.position = Vec3::new(0.0, 0.0, 0.0);
                next.second.orientation = Quaternion::rotation_x(0.0);

                next.foot_l.position = Vec3::new(
                    -skeleton_attr.foot.0,
                    skeleton_attr.foot.1 + slow * -3.0 + quick * 3.0 - 4.0,
                    skeleton_attr.foot.2 + skeleton_attr.foot.2,
                );
                next.foot_l.orientation = Quaternion::rotation_x(slow * 0.6)
                    * Quaternion::rotation_y((slow * -0.2).max(0.0));
                next.foot_l.scale = Vec3::one();

                next.foot_r.position = Vec3::new(
                    skeleton_attr.foot.0,
                    skeleton_attr.foot.1 + slow * 3.0 + quick * -3.0 + 5.0,
                    skeleton_attr.foot.2 + skeleton_attr.foot.2,
                );
                next.foot_r.orientation = Quaternion::rotation_x(slow * -0.6)
                    * Quaternion::rotation_y((slow * 0.2).min(0.0));
                next.foot_r.scale = Vec3::one();

                next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                next.torso.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 8.0 * 1.01;
            },
            Some(ToolKind::Axe(_)) => {
                next.head.position =
                    Vec3::new(0.0, 0.0 + skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.orientation = Quaternion::rotation_z(0.1 + axe * 0.2)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.2);
                next.head.scale = Vec3::one() * 1.01;

                next.upper_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.upper_torso.0,
                    skeleton_attr.upper_torso.1,
                );
                next.upper_torso.orientation = Quaternion::rotation_z(0.2 + axe * 0.2);
                next.upper_torso.scale = Vec3::one();

                next.lower_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.lower_torso.0,
                    skeleton_attr.lower_torso.1,
                );
                next.lower_torso.orientation = Quaternion::rotation_z(0.2 + axe * -0.2);

                next.hand_l.position = Vec3::new(-0.5, 0.0, 4.0);
                next.hand_l.orientation = Quaternion::rotation_x(PI / 2.0)
                    * Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_y(0.0);
                next.hand_l.scale = Vec3::one() * 1.08;
                next.hand_r.position = Vec3::new(0.5, 0.0, -2.5);
                next.hand_r.orientation = Quaternion::rotation_x(PI / 2.0)
                    * Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_y(0.0);
                next.hand_r.scale = Vec3::one() * 1.06;
                next.main.position = Vec3::new(-0.0, -2.0, -1.0);
                next.main.orientation = Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);

                next.control.position = Vec3::new(2.0 + axe * -7.0, 11.0, 3.0);
                next.control.orientation = Quaternion::rotation_x(1.6)
                    * Quaternion::rotation_y(-2.0 + axe * 0.5)
                    * Quaternion::rotation_z(PI * 0.4);

                next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                next.torso.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 8.0 * 1.01;
            },
            Some(ToolKind::Hammer(_)) => {
                next.hand_l.position = Vec3::new(
                    -skeleton_attr.hand.0 - 7.0,
                    skeleton_attr.hand.1 - 7.0,
                    skeleton_attr.hand.2 + 10.0,
                );
                next.hand_l.orientation =
                    Quaternion::rotation_x(0.57) * Quaternion::rotation_z(1.57);
                next.hand_l.scale = Vec3::one() * 1.02;

                next.hand_r.position = Vec3::new(
                    skeleton_attr.hand.0 - 7.0,
                    skeleton_attr.hand.1 - 7.0,
                    skeleton_attr.hand.2 + 10.0,
                );
                next.hand_r.orientation =
                    Quaternion::rotation_x(0.57) * Quaternion::rotation_z(1.57);
                next.hand_r.scale = Vec3::one() * 1.02;

                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(-1.57)
                    * Quaternion::rotation_z(1.0);
                next.main.scale = Vec3::one() * 1.02;

                next.head.position =
                    Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1);
                next.head.orientation = Quaternion::rotation_z(slower * 0.03)
                    * Quaternion::rotation_x(-0.3 + slowersmooth * 0.1)
                    * Quaternion::rotation_y(slower * 0.05 + slowersmooth * 0.06)
                    * Quaternion::rotation_z((slowersmooth * -0.4).max(0.0));
                next.head.scale = Vec3::one() * 1.01;

                next.upper_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.upper_torso.0,
                    skeleton_attr.upper_torso.1,
                );
                next.upper_torso.orientation =
                    Quaternion::rotation_z(slower * 0.18 + slowersmooth * 0.15)
                        * Quaternion::rotation_x(0.0 + slower * 0.18 + slowersmooth * 0.15)
                        * Quaternion::rotation_y(0.0);

                next.lower_torso.position = Vec3::new(
                    0.0,
                    skeleton_attr.lower_torso.0,
                    skeleton_attr.lower_torso.1,
                );
                next.lower_torso.orientation =
                    Quaternion::rotation_z(slower * -0.1 + slowersmooth * -0.075)
                        * Quaternion::rotation_x(0.0 + slower * -0.1)
                        * Quaternion::rotation_y(slower * -0.1);

                next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                next.torso.orientation = Quaternion::rotation_z(0.0);
                next.torso.scale = Vec3::one() / 8.0 * 1.01;

                if velocity > 0.5 {
                    next.foot_l.position =
                        Vec3::new(-skeleton_attr.foot.0, foot * -6.0, skeleton_attr.foot.2);
                    next.foot_l.orientation = Quaternion::rotation_x(foot * -0.4)
                        * Quaternion::rotation_z((slower * 0.3).max(0.0));
                    next.foot_l.scale = Vec3::one();

                    next.foot_r.position =
                        Vec3::new(skeleton_attr.foot.0, foot * 6.0, skeleton_attr.foot.2);
                    next.foot_r.orientation = Quaternion::rotation_x(foot * 0.4)
                        * Quaternion::rotation_z((slower * 0.3).max(0.0));
                    next.foot_r.scale = Vec3::one();
                    next.torso.position = Vec3::new(0.0, 0.0, 0.) * 1.01;
                    next.torso.orientation =
                        Quaternion::rotation_z(0.0) * Quaternion::rotation_x(-0.15);
                    next.torso.scale = Vec3::one() / 8.0 * 1.01;
                } else {
                    next.foot_l.position = Vec3::new(
                        -skeleton_attr.foot.0,
                        -2.5,
                        skeleton_attr.foot.2 + (slower * 2.5).max(0.0),
                    );
                    next.foot_l.orientation = Quaternion::rotation_x(slower * -0.2 - 0.2)
                        * Quaternion::rotation_z((slower * 1.0).max(0.0));
                    next.foot_l.scale = Vec3::one();

                    next.foot_r.position = Vec3::new(
                        skeleton_attr.foot.0,
                        3.5 - slower * 2.0,
                        skeleton_attr.foot.2,
                    );
                    next.foot_r.orientation = Quaternion::rotation_x(slower * 0.1)
                        * Quaternion::rotation_z((slower * 0.5).max(0.0));
                    next.foot_r.scale = Vec3::one();
                    next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                    next.torso.orientation = Quaternion::rotation_z(0.0);
                    next.torso.scale = Vec3::one() / 8.0 * 1.01;
                }

                next.control.scale = Vec3::one();
                next.control.position = Vec3::new(-8.0, 7.0 + slower * 4.0, 1.0 + slower * -9.0);
                next.control.orientation = Quaternion::rotation_x(-1.5 + slower * -1.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(1.5);
                next.control.scale = Vec3::one();

                next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                next.torso.orientation = Quaternion::rotation_z(0.0);
                next.torso.scale = Vec3::one() / 8.0 * 1.01;
            },
            Some(ToolKind::Staff(_)) => {
                next.head.orientation =
                    Quaternion::rotation_x(staff * 0.2) * Quaternion::rotation_z(staff * 0.2);
                next.hand_l.position = Vec3::new(11.0, 5.0, -4.0);
                next.hand_l.orientation =
                    Quaternion::rotation_x(1.27) * Quaternion::rotation_y(0.0);
                next.hand_l.scale = Vec3::one() * 1.02;
                next.hand_r.position = Vec3::new(12.0, 5.5, 2.0);
                next.hand_r.orientation =
                    Quaternion::rotation_x(1.57) * Quaternion::rotation_y(0.2);
                next.hand_r.scale = Vec3::one() * 1.02;
                next.main.position = Vec3::new(12.0, 8.5, 13.2);
                next.main.orientation = Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(3.14)
                    * Quaternion::rotation_z(0.0);
                next.upper_torso.orientation = Quaternion::rotation_z(staff * 0.3);
                next.lower_torso.orientation = Quaternion::rotation_z(staff * 0.4);

                next.control.position = Vec3::new(-20.0, 5.0 + staff * 3.0, 1.0);
                next.control.orientation = Quaternion::rotation_x(staff * 1.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Shield(_)) => {
                next.head.position = Vec3::new(
                    0.0,
                    0.0 + skeleton_attr.head.0 + decel * 0.8,
                    skeleton_attr.head.1,
                );
                next.head.orientation = Quaternion::rotation_z(decel * 0.25)
                    * Quaternion::rotation_x(0.0 + decel * 0.1)
                    * Quaternion::rotation_y(decel * -0.1);
                next.head.scale = Vec3::one() * 1.01;

                next.upper_torso.position = Vec3::new(0.0, 0.0, 7.0);
                next.upper_torso.orientation = Quaternion::rotation_z(decel * -0.2)
                    * Quaternion::rotation_x(0.0 + decel * -0.2)
                    * Quaternion::rotation_y(decel * 0.2);

                next.torso.position = Vec3::new(0.0, 0.0, 0.0) * 1.01;
                next.torso.scale = Vec3::one() / 8.0 * 1.01;

                next.lower_torso.position = Vec3::new(0.0, 0.0, 0.0);

                next.hand_l.position = Vec3::new(0.0, 0.0, 0.0);
                next.hand_l.orientation = Quaternion::rotation_x(0.0);
                next.hand_l.scale = Vec3::one() * 1.01;

                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_z(0.0);

                next.hand_r.position = Vec3::new(0.0, 0.0, 0.0);
                next.hand_r.orientation = Quaternion::rotation_x(0.0);
                next.hand_r.scale = Vec3::one() * 1.01;

                next.second.position = Vec3::new(0.0, 0.0, 0.0);
                next.second.orientation = Quaternion::rotation_x(0.0);
            },
            Some(ToolKind::Debug(_)) => {
                next.hand_l.position = Vec3::new(-7.0, 4.0, 3.0);
                next.hand_l.orientation = Quaternion::rotation_x(1.27)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.hand_l.scale = Vec3::one() * 1.01;
                next.main.position = Vec3::new(-5.0, 5.0, 23.0);
                next.main.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(PI)
                    * Quaternion::rotation_y(0.0);
                next.main.scale = Vec3::one();
                next.torso.position = Vec3::new(0.0, 0.0, 0.1) * 1.01;
                next.torso.orientation = Quaternion::rotation_x(0.0);
                next.torso.scale = Vec3::one() / 8.0 * 1.01;
            },
            Some(ToolKind::NpcWeapon(_)) => {
                if switch > 0.0 {
                    next.head.position =
                        Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1) * 1.02;
                    next.head.orientation = Quaternion::rotation_z((twist * -0.5).max(-1.0))
                        * Quaternion::rotation_x(-0.2);
                    next.head.scale = Vec3::one() * 1.02;

                    next.upper_torso.position = Vec3::new(
                        0.0,
                        skeleton_attr.upper_torso.0,
                        skeleton_attr.upper_torso.1,
                    );
                    next.upper_torso.orientation = Quaternion::rotation_z((twist * 0.5).min(1.0))
                        * Quaternion::rotation_x(0.0);
                    next.upper_torso.scale = Vec3::one();

                    next.lower_torso.position = Vec3::new(
                        0.0,
                        skeleton_attr.lower_torso.0,
                        skeleton_attr.lower_torso.1,
                    );
                    next.lower_torso.orientation = Quaternion::rotation_z((twist * -0.5).max(-1.0))
                        * Quaternion::rotation_x(0.0);
                    next.lower_torso.scale = Vec3::one();

                    next.hand_r.position = Vec3::new(
                        skeleton_attr.hand.0,
                        skeleton_attr.hand.1,
                        skeleton_attr.hand.2,
                    );
                    next.hand_r.orientation =
                        Quaternion::rotation_z(-1.5) * Quaternion::rotation_x(0.0);
                    next.hand_r.scale = Vec3::one() * 1.02;

                    next.arm_control_r.position = Vec3::new(0.0, 0.0, -4.0);
                    next.arm_control_r.orientation = Quaternion::rotation_x(1.0)
                        * Quaternion::rotation_y(slow * -1.35)
                        * Quaternion::rotation_z(-0.0);
                    next.arm_control_r.scale = Vec3::one() * 1.02;

                    next.tail.orientation = Quaternion::rotation_z(twist * 0.5);
                } else {
                    next.head.position =
                        Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1) * 1.02;
                    next.head.orientation = Quaternion::rotation_z((twist * 0.5).min(1.0))
                        * Quaternion::rotation_x(-0.2);
                    next.head.scale = Vec3::one() * 1.02;

                    next.upper_torso.position = Vec3::new(
                        0.0,
                        skeleton_attr.upper_torso.0,
                        skeleton_attr.upper_torso.1,
                    );
                    next.upper_torso.orientation = Quaternion::rotation_z((twist * -0.5).max(-1.0))
                        * Quaternion::rotation_x(0.0);
                    next.upper_torso.scale = Vec3::one();

                    next.lower_torso.position = Vec3::new(
                        0.0,
                        skeleton_attr.lower_torso.0,
                        skeleton_attr.lower_torso.1,
                    );
                    next.lower_torso.orientation = Quaternion::rotation_z((twist * 0.5).min(1.0))
                        * Quaternion::rotation_x(0.0);
                    next.lower_torso.scale = Vec3::one();

                    next.arm_control_l.position = Vec3::new(0.0, 0.0, -4.0);
                    next.arm_control_l.orientation = Quaternion::rotation_x(1.0)
                        * Quaternion::rotation_y(slow * 1.35)
                        * Quaternion::rotation_z(0.0);
                    next.arm_control_l.scale = Vec3::one() * 1.02;

                    next.hand_l.position = Vec3::new(
                        -skeleton_attr.hand.0,
                        skeleton_attr.hand.1,
                        skeleton_attr.hand.2,
                    );
                    next.hand_l.orientation =
                        Quaternion::rotation_z(1.5) * Quaternion::rotation_x(0.0);
                    next.hand_l.scale = Vec3::one() * 1.02;

                    next.tail.orientation = Quaternion::rotation_z(twist * -0.5);
                };
            },
            _ => {},
        }
        next.shoulder_l.scale = Vec3::one();
        next.shoulder_r.scale = Vec3::one();

        next.second.scale = match (
            active_tool_kind.map(|tk| tk.hands()),
            second_tool_kind.map(|tk| tk.hands()),
        ) {
            (Some(Hands::OneHand), Some(Hands::OneHand)) => Vec3::one(),
            (_, _) => Vec3::zero(),
        };

        next
    }
}
