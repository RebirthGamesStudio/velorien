use super::{
    super::{vek::*, Animation},
    BipedLargeSkeleton, SkeletonAttr,
};
use common::{
    comp::item::{Hands, ToolKind},
    states::utils::StageSection,
};
use std::f32::consts::PI;

pub struct SpinMeleeAnimation;

impl Animation for SpinMeleeAnimation {
    type Dependency = (
        Option<ToolKind>,
        Option<ToolKind>,
        Vec3<f32>,
        f64,
        Option<StageSection>,
    );
    type Skeleton = BipedLargeSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"biped_large_spinmelee\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "biped_large_spinmelee")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, second_tool_kind, velocity, _global_time, stage_section): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let lab = 1.0;
        let speed = Vec2::<f32>::from(velocity).magnitude();
        let mut next = (*skeleton).clone();
        //torso movement
        let xshift = if velocity.z.abs() < 0.1 {
            ((anim_time as f32 - 1.1) * lab as f32 * 3.0).sin()
        } else {
            0.0
        };
        let yshift = if velocity.z.abs() < 0.1 {
            ((anim_time as f32 - 1.1) * lab as f32 * 3.0 + PI / 2.0).sin()
        } else {
            0.0
        };

        let spin = if anim_time < 1.1 && velocity.z.abs() < 0.1 {
            0.5 * ((anim_time as f32).powf(2.0))
        } else {
            lab as f32 * anim_time as f32 * 0.9
        };
        let movement = anim_time as f32 * 1.0;

        //feet
        let slowersmooth = (anim_time as f32 * lab as f32 * 4.0).sin();
        let quick = (anim_time as f32 * lab as f32 * 8.0).sin();

        match active_tool_kind {
            Some(ToolKind::Sword(_)) => {
                next.hand_l.position = Vec3::new(-0.75, -1.0, 2.5);
                next.hand_l.orientation =
                    Quaternion::rotation_x(1.47) * Quaternion::rotation_y(-0.2);
                next.hand_l.scale = Vec3::one() * 1.02;
                next.hand_r.position = Vec3::new(0.75, -1.5, -0.5);
                next.hand_r.orientation =
                    Quaternion::rotation_x(1.47) * Quaternion::rotation_y(0.3);
                next.hand_r.scale = Vec3::one() * 1.02;
                next.main.position = Vec3::new(0.0, 5.0, 2.0);
                next.main.orientation = Quaternion::rotation_x(-0.1)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.head.position = Vec3::new(0.0, s_a.head.0 + 0.0, s_a.head.1);

                if let Some(stage_section) = stage_section {
                    match stage_section {
                        StageSection::Buildup => {
                            next.control.position =
                                Vec3::new(-7.0, 7.0 + movement * -8.0, 2.0 + movement * -6.0);
                            next.control.orientation = Quaternion::rotation_x(movement * -0.5)
                                * Quaternion::rotation_y(movement * 0.3)
                                * Quaternion::rotation_z(movement * -1.5);
                            next.upper_torso.position = Vec3::new(
                                0.0,
                                s_a.upper_torso.0 + movement * -1.0,
                                s_a.upper_torso.1 + movement * -2.5,
                            );
                            next.upper_torso.orientation = Quaternion::rotation_x(movement * -1.1)
                                * Quaternion::rotation_z(movement * -0.35);
                            next.lower_torso.orientation = Quaternion::rotation_z(movement * 0.5);
                            next.head.position = Vec3::new(
                                0.0,
                                s_a.head.0 - 2.0 + movement * -6.0,
                                s_a.head.1 + movement * -4.0,
                            );
                            next.head.orientation = Quaternion::rotation_x(movement * 0.9)
                                * Quaternion::rotation_y(0.0)
                                * Quaternion::rotation_z(movement * 0.05);

                            next.foot_l.position =
                                Vec3::new(-s_a.foot.0, s_a.foot.1 + movement * 4.0, s_a.foot.2);
                            next.foot_l.orientation = Quaternion::rotation_x(movement * 0.2);
                            next.foot_r.position = Vec3::new(
                                s_a.foot.0,
                                s_a.foot.1 + movement * -12.0,
                                s_a.foot.2 + movement * 1.0 + quick * 1.0,
                            );
                            next.foot_r.orientation = Quaternion::rotation_x(movement * -1.0)
                                * Quaternion::rotation_z(movement * -0.8);
                        },
                        StageSection::Swing => {
                            next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);

                            next.control.position = Vec3::new(-7.0, 7.0, 2.0);
                            next.control.orientation = Quaternion::rotation_x(-PI / 2.0)
                                * Quaternion::rotation_z(-PI / 2.0);
                            next.torso.orientation = Quaternion::rotation_z(movement * PI * 2.0);

                            next.upper_torso.position =
                                Vec3::new(0.0, s_a.upper_torso.0, s_a.upper_torso.1);
                            next.upper_torso.orientation = Quaternion::rotation_y(0.3);
                            next.head.position = Vec3::new(0.0, s_a.head.0 - 2.0, s_a.head.1);
                            next.head.orientation = Quaternion::rotation_x(-0.15);
                            next.lower_torso.orientation = Quaternion::rotation_x(0.2);
                        },
                        StageSection::Recover => {
                            next.head.position = Vec3::new(0.0, s_a.head.0 - 2.0, s_a.head.1);
                            next.control.position = Vec3::new(-7.0, 7.0, 2.0);
                            next.control.orientation =
                                Quaternion::rotation_x(-PI / 2.0 + movement * PI / 2.0)
                                    * Quaternion::rotation_z(-PI / 2.0 + movement * PI / 2.0);
                            next.head.orientation = Quaternion::rotation_x(-0.15 + movement * 0.15);
                            next.upper_torso.orientation =
                                Quaternion::rotation_y(0.3 + movement * -0.3)
                        },
                        _ => {},
                    }
                }
            },
            Some(ToolKind::Axe(_)) => {
                next.hand_l.position = Vec3::new(-0.5, 0.0, 4.0);
                next.hand_l.orientation = Quaternion::rotation_x(PI / 2.0)
                    * Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_y(PI);
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

                next.control.position = Vec3::new(0.0, 16.0, 3.0);
                next.control.orientation = Quaternion::rotation_x(-1.4)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(1.4);
                next.control.scale = Vec3::one();

                next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);
                next.head.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(-0.15)
                    * Quaternion::rotation_y(0.08);
                next.upper_torso.position =
                    Vec3::new(0.0, s_a.upper_torso.0 - 3.0, s_a.upper_torso.1 - 2.0);
                next.upper_torso.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(-0.1)
                    * Quaternion::rotation_y(0.3);
                next.upper_torso.scale = Vec3::one();

                next.lower_torso.position = Vec3::new(0.0, 3.0, -2.5);
                next.lower_torso.orientation = Quaternion::rotation_z(0.0)
                    * Quaternion::rotation_x(0.7)
                    * Quaternion::rotation_y(0.0);
                next.lower_torso.scale = Vec3::one();
                next.torso.position = Vec3::new(
                    -xshift * (anim_time as f32).min(0.6),
                    -yshift * (anim_time as f32).min(0.6),
                    0.0,
                ) * 1.01;
                next.torso.orientation = Quaternion::rotation_z(spin * -16.0)
                    * Quaternion::rotation_x(0.0)
                    * Quaternion::rotation_y(0.0);
                next.torso.scale = Vec3::one() / 11.0 * 1.01;
                if velocity.z.abs() > 0.1 {
                    next.foot_l.position = Vec3::new(-s_a.foot.0, 8.0, s_a.foot.2 + 2.0);
                    next.foot_l.orientation =
                        Quaternion::rotation_x(1.0) * Quaternion::rotation_z(0.0);
                    next.foot_l.scale = Vec3::one();

                    next.foot_r.position = Vec3::new(s_a.foot.0, 8.0, s_a.foot.2 + 2.0);
                    next.foot_r.orientation = Quaternion::rotation_x(1.0);
                    next.foot_r.scale = Vec3::one();
                } else if speed < 0.5 {
                    next.foot_l.position = Vec3::new(-s_a.foot.0, 2.0 + quick * -6.0, s_a.foot.2);
                    next.foot_l.orientation = Quaternion::rotation_x(0.5 + slowersmooth * 0.2)
                        * Quaternion::rotation_z(0.0);
                    next.foot_l.scale = Vec3::one();

                    next.foot_r.position = Vec3::new(s_a.foot.0, 4.0, s_a.foot.2);
                    next.foot_r.orientation = Quaternion::rotation_x(0.5 - slowersmooth * 0.2)
                        * Quaternion::rotation_y(-0.4);
                    next.foot_r.scale = Vec3::one();
                } else {
                    next.foot_l.position = Vec3::new(-s_a.foot.0, 2.0 + quick * -6.0, s_a.foot.2);
                    next.foot_l.orientation = Quaternion::rotation_x(0.5 + slowersmooth * 0.2)
                        * Quaternion::rotation_z(0.0);
                    next.foot_l.scale = Vec3::one();

                    next.foot_r.position = Vec3::new(s_a.foot.0, 2.0 + quick * 6.0, s_a.foot.2);
                    next.foot_r.orientation = Quaternion::rotation_x(0.5 - slowersmooth * 0.2)
                        * Quaternion::rotation_z(0.0);
                    next.foot_r.scale = Vec3::one();
                };
            },
            _ => {},
        }

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
