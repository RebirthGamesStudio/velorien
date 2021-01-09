pub mod feed;
pub mod fly;
pub mod idle;
pub mod run;

// Reexports
pub use self::{feed::FeedAnimation, fly::FlyAnimation, idle::IdleAnimation, run::RunAnimation};

use super::{make_bone, vek::*, FigureBoneData, Skeleton};
use common::comp::{self};
use core::convert::TryFrom;

pub type Body = comp::bird_medium::Body;

skeleton_impls!(struct BirdMediumSkeleton {
    + head,
    + torso,
    + tail,
    + wing_l,
    + wing_r,
    + leg_l,
    + leg_r,
});

impl Skeleton for BirdMediumSkeleton {
    type Attr = SkeletonAttr;
    type Body = Body;

    const BONE_COUNT: usize = 7;
    #[cfg(feature = "use-dyn-lib")]
    const COMPUTE_FN: &'static [u8] = b"bird_medium_compute_mats\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "bird_medium_compute_mats")]

    fn compute_matrices_inner(
        &self,
        base_mat: Mat4<f32>,
        buf: &mut [FigureBoneData; super::MAX_BONE_COUNT],
    ) -> Vec3<f32> {
        let torso_mat = base_mat * Mat4::<f32>::from(self.torso);

        *(<&mut [_; Self::BONE_COUNT]>::try_from(&mut buf[0..Self::BONE_COUNT]).unwrap()) = [
            make_bone(torso_mat * Mat4::<f32>::from(self.head)),
            make_bone(torso_mat),
            make_bone(torso_mat * Mat4::<f32>::from(self.tail)),
            make_bone(torso_mat * Mat4::<f32>::from(self.wing_l)),
            make_bone(torso_mat * Mat4::<f32>::from(self.wing_r)),
            make_bone(base_mat * Mat4::<f32>::from(self.leg_l)),
            make_bone(base_mat * Mat4::<f32>::from(self.leg_r)),
        ];
        Vec3::default()
    }
}

pub struct SkeletonAttr {
    head: (f32, f32),
    chest: (f32, f32),
    tail: (f32, f32),
    wing: (f32, f32, f32),
    foot: (f32, f32, f32),
    feed: f32,
}

impl<'a> std::convert::TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::BirdMedium(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            head: (0.0, 0.0),
            chest: (0.0, 0.0),
            tail: (0.0, 0.0),
            wing: (0.0, 0.0, 0.0),
            foot: (0.0, 0.0, 0.0),
            feed: 0.0,
        }
    }
}

impl<'a> From<&'a Body> for SkeletonAttr {
    fn from(body: &'a Body) -> Self {
        use comp::bird_medium::Species::*;
        Self {
            head: match (body.species, body.body_type) {
                (Duck, _) => (4.0, 3.0),
                (Chicken, _) => (4.0, 3.0),
                (Goose, _) => (5.0, 2.5),
                (Peacock, _) => (1.0, 1.0),
                (Eagle, _) => (2.5, 5.0),
                (Snowyowl, _) => (2.5, 5.0),
                (Parrot, _) => (0.5, 4.5),
                (Cockatrice, _) => (0.0, 4.0),
            },
            chest: match (body.species, body.body_type) {
                (Duck, _) => (0.0, 5.0),
                (Chicken, _) => (0.0, 5.0),
                (Goose, _) => (0.0, 8.0),
                (Peacock, _) => (0.0, 10.0),
                (Eagle, _) => (0.0, 8.0),
                (Snowyowl, _) => (0.0, 4.5),
                (Parrot, _) => (0.0, 5.0),
                (Cockatrice, _) => (0.0, 12.5),
            },
            tail: match (body.species, body.body_type) {
                (Duck, _) => (-3.0, 1.5),
                (Chicken, _) => (-3.0, 1.5),
                (Goose, _) => (-5.0, 3.0),
                (Peacock, _) => (-5.5, 2.0),
                (Eagle, _) => (-8.0, -4.0),
                (Snowyowl, _) => (-6.0, -2.0),
                (Parrot, _) => (-8.0, -2.0),
                (Cockatrice, _) => (-10.0, -2.5),
            },
            wing: match (body.species, body.body_type) {
                (Duck, _) => (2.75, 0.0, 1.0),
                (Chicken, _) => (2.75, 0.0, 1.0),
                (Goose, _) => (3.75, -1.0, 2.0),
                (Peacock, _) => (3.0, 0.0, 1.0),
                (Eagle, _) => (3.0, -8.0, 4.0),
                (Snowyowl, _) => (3.5, -5.5, 4.0),
                (Parrot, _) => (2.0, -4.5, 3.0),
                (Cockatrice, _) => (4.5, -2.5, 1.5),
            },
            foot: match (body.species, body.body_type) {
                (Duck, _) => (2.0, -1.5, 4.0),
                (Chicken, _) => (2.0, -1.5, 4.0),
                (Goose, _) => (2.0, -1.5, 7.0),
                (Peacock, _) => (2.0, -2.5, 8.0),
                (Eagle, _) => (2.0, -2.0, 8.0),
                (Snowyowl, _) => (1.5, -2.5, 7.0),
                (Parrot, _) => (1.5, -3.0, 3.0),
                (Cockatrice, _) => (4.0, -3.5, 12.0),
            },
            feed: match (body.species, body.body_type) {
                (Goose, _) => 1.4,
                (Peacock, _) => 1.6,
                (Eagle, _) => 1.2,
                (Parrot, _) => 1.2,
                (Cockatrice, _) => 1.3,
                _ => 1.0,
            },
        }
    }
}
