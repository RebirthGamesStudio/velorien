use common::{terrain::TerrainGrid, vol::ReadVol};
use common_base::span;
use std::f32::consts::PI;
use treeculler::Frustum;
use vek::*;

pub const NEAR_PLANE: f32 = 0.25;
pub const FAR_PLANE: f32 = 100000.0;

const FIRST_PERSON_INTERP_TIME: f32 = 0.1;
const THIRD_PERSON_INTERP_TIME: f32 = 0.1;
const FREEFLY_INTERP_TIME: f32 = 0.0;
const LERP_ORI_RATE: f32 = 15.0;
pub const MIN_ZOOM: f32 = 0.1;

// Possible TODO: Add more modes
#[derive(PartialEq, Clone, Copy, Eq, Hash)]
pub enum CameraMode {
    FirstPerson = 0,
    ThirdPerson = 1,
    Freefly = 2,
}

impl Default for CameraMode {
    fn default() -> Self { Self::ThirdPerson }
}

#[derive(Clone, Copy)]
pub struct Dependents {
    pub view_mat: Mat4<f32>,
    pub view_mat_inv: Mat4<f32>,
    pub proj_mat: Mat4<f32>,
    pub proj_mat_inv: Mat4<f32>,
    pub cam_pos: Vec3<f32>,
    pub cam_dir: Vec3<f32>,
}

pub struct Camera {
    tgt_focus: Vec3<f32>,
    focus: Vec3<f32>,
    tgt_ori: Vec3<f32>,
    ori: Vec3<f32>,
    tgt_dist: f32,
    dist: f32,
    tgt_fov: f32,
    fov: f32,
    aspect: f32,
    mode: CameraMode,

    last_time: Option<f64>,

    dependents: Dependents,
    frustum: Frustum<f32>,
}

fn clamp_and_modulate(ori: Vec3<f32>) -> Vec3<f32> {
    Vec3 {
        // Wrap camera yaw
        x: ori.x.rem_euclid(2.0 * PI),
        // Clamp camera pitch to the vertical limits
        y: ori.y.min(PI / 2.0 - 0.0001).max(-PI / 2.0 + 0.0001),
        // Wrap camera roll
        z: ori.z.rem_euclid(2.0 * PI),
    }
}

impl Camera {
    /// Create a new `Camera` with default parameters.
    pub fn new(aspect: f32, mode: CameraMode) -> Self {
        // Make sure aspect is valid
        let aspect = if aspect.is_normal() { aspect } else { 1.0 };

        Self {
            tgt_focus: Vec3::unit_z() * 10.0,
            focus: Vec3::unit_z() * 10.0,
            tgt_ori: Vec3::zero(),
            ori: Vec3::zero(),
            tgt_dist: 10.0,
            dist: 10.0,
            tgt_fov: 1.1,
            fov: 1.1,
            aspect,
            mode,

            last_time: None,

            dependents: Dependents {
                view_mat: Mat4::identity(),
                view_mat_inv: Mat4::identity(),
                proj_mat: Mat4::identity(),
                proj_mat_inv: Mat4::identity(),
                cam_pos: Vec3::zero(),
                cam_dir: Vec3::unit_y(),
            },
            frustum: Frustum::from_modelview_projection(Mat4::identity().into_col_arrays()),
        }
    }

    /// Compute the transformation matrices (view matrix and projection matrix)
    /// and position of the camera.
    pub fn compute_dependents(&mut self, terrain: &TerrainGrid) {
        self.compute_dependents_full(terrain, |block| !block.is_opaque())
    }

    /// The is_fluid argument should return true for transparent voxels.
    pub fn compute_dependents_full<V: ReadVol>(
        &mut self,
        terrain: &V,
        is_transparent: fn(&V::Vox) -> bool,
    ) {
        span!(_guard, "compute_dependents", "Camera::compute_dependents");
        let dist = {
            let (start, end) = (self.focus - self.forward() * self.dist, self.focus);

            match terrain
                .ray(start, end)
                .ignore_error()
                .max_iter(500)
                .until(is_transparent)
                .cast()
            {
                (d, Ok(Some(_))) => f32::min(self.dist - d - 0.03, self.dist),
                (_, Ok(None)) => self.dist,
                (_, Err(_)) => self.dist,
            }
            .max(0.0)
        };

        self.dependents.view_mat = Mat4::<f32>::identity()
            * Mat4::translation_3d(-Vec3::unit_z() * dist)
            * Mat4::rotation_z(self.ori.z)
            * Mat4::rotation_x(self.ori.y)
            * Mat4::rotation_y(self.ori.x)
            * Mat4::rotation_3d(PI / 2.0, -Vec4::unit_x())
            * Mat4::translation_3d(-self.focus.map(|e| e.fract()));
        self.dependents.view_mat_inv = self.dependents.view_mat.inverted();

        self.dependents.proj_mat =
            Mat4::perspective_rh_zo(self.fov, self.aspect, NEAR_PLANE, FAR_PLANE);
        self.dependents.proj_mat_inv = self.dependents.proj_mat.inverted();

        // TODO: Make this more efficient.
        self.dependents.cam_pos = Vec3::from(self.dependents.view_mat.inverted() * Vec4::unit_w());
        self.frustum = Frustum::from_modelview_projection(
            (self.dependents.proj_mat
                * self.dependents.view_mat
                * Mat4::translation_3d(-self.focus.map(|e| e.trunc())))
            .into_col_arrays(),
        );

        self.dependents.cam_dir = Vec3::from(self.dependents.view_mat.inverted() * -Vec4::unit_z());
    }

    pub fn frustum(&self) -> &Frustum<f32> { &self.frustum }

    pub fn dependents(&self) -> Dependents { self.dependents }

    /// Rotate the camera about its focus by the given delta, limiting the input
    /// accordingly.
    pub fn rotate_by(&mut self, delta: Vec3<f32>) {
        // Wrap camera yaw
        self.tgt_ori.x = (self.tgt_ori.x + delta.x).rem_euclid(2.0 * PI);
        // Clamp camera pitch to the vertical limits
        self.tgt_ori.y = (self.tgt_ori.y + delta.y)
            .min(PI / 2.0 - 0.0001)
            .max(-PI / 2.0 + 0.0001);
        // Wrap camera roll
        self.tgt_ori.z = (self.tgt_ori.z + delta.z).rem_euclid(2.0 * PI);
    }

    /// Set the orientation of the camera about its focus.
    pub fn set_orientation(&mut self, ori: Vec3<f32>) { self.tgt_ori = clamp_and_modulate(ori); }

    /// Set the orientation of the camera about its focus without lerping.
    pub fn set_orientation_instant(&mut self, ori: Vec3<f32>) {
        self.set_orientation(ori);
        self.ori = self.tgt_ori;
    }

    /// Zoom the camera by the given delta, limiting the input accordingly.
    pub fn zoom_by(&mut self, delta: f32) {
        if self.mode == CameraMode::ThirdPerson {
            // Clamp camera dist to the 2 <= x <= infinity range
            self.tgt_dist = (self.tgt_dist + delta).max(2.0);
        }
    }

    /// Zoom with the ability to switch between first and third-person mode.
    pub fn zoom_switch(&mut self, delta: f32) {
        if delta > 0_f32 || self.mode != CameraMode::FirstPerson {
            let t = self.tgt_dist + delta;
            const MIN_THIRD_PERSON: f32 = 2.35;
            match self.mode {
                CameraMode::ThirdPerson => {
                    if t < MIN_THIRD_PERSON {
                        self.set_mode(CameraMode::FirstPerson);
                    } else {
                        self.tgt_dist = t;
                    }
                },
                CameraMode::FirstPerson => {
                    self.set_mode(CameraMode::ThirdPerson);
                    self.tgt_dist = MIN_THIRD_PERSON;
                },
                _ => {},
            }
        }
    }

    /// Get the distance of the camera from the focus
    pub fn get_distance(&self) -> f32 { self.dist }

    /// Set the distance of the camera from the focus (i.e., zoom).
    pub fn set_distance(&mut self, dist: f32) { self.tgt_dist = dist; }

    pub fn update(&mut self, time: f64, dt: f32, smoothing_enabled: bool) {
        // This is horribly frame time dependent, but so is most of the game
        let delta = self.last_time.replace(time).map_or(0.0, |t| time - t);
        if (self.dist - self.tgt_dist).abs() > 0.01 {
            self.dist = f32::lerp(
                self.dist,
                self.tgt_dist,
                0.65 * (delta as f32) / self.interp_time(),
            );
        }

        if (self.fov - self.tgt_fov).abs() > 0.01 {
            self.fov = f32::lerp(
                self.fov,
                self.tgt_fov,
                0.65 * (delta as f32) / self.interp_time(),
            );
        }

        if (self.focus - self.tgt_focus).magnitude_squared() > 0.001 {
            let lerped_focus = Lerp::lerp(
                self.focus,
                self.tgt_focus,
                (delta as f32) / self.interp_time()
                    * if matches!(self.mode, CameraMode::FirstPerson) {
                        2.0
                    } else {
                        1.0
                    },
            );

            self.focus.x = lerped_focus.x;
            self.focus.y = lerped_focus.y;

            // Always lerp in z
            self.focus.z = lerped_focus.z;
        }

        let lerp_angle = |a: f32, b: f32, rate: f32| {
            let offs = [-2.0 * PI, 0.0, 2.0 * PI]
                .iter()
                .min_by_key(|offs: &&f32| ((a - (b + *offs)).abs() * 1000.0) as i32)
                .unwrap();
            Lerp::lerp(a, b + *offs, rate)
        };

        let ori = if smoothing_enabled {
            Vec3::new(
                lerp_angle(self.ori.x, self.tgt_ori.x, LERP_ORI_RATE * dt),
                Lerp::lerp(self.ori.y, self.tgt_ori.y, LERP_ORI_RATE * dt),
                lerp_angle(self.ori.z, self.tgt_ori.z, LERP_ORI_RATE * dt),
            )
        } else {
            self.tgt_ori
        };
        self.ori = clamp_and_modulate(ori);
    }

    pub fn interp_time(&self) -> f32 {
        match self.mode {
            CameraMode::FirstPerson => FIRST_PERSON_INTERP_TIME,
            CameraMode::ThirdPerson => THIRD_PERSON_INTERP_TIME,
            CameraMode::Freefly => FREEFLY_INTERP_TIME,
        }
    }

    /// Get the focus position of the camera.
    pub fn get_focus_pos(&self) -> Vec3<f32> { self.focus }

    /// Set the focus position of the camera.
    pub fn set_focus_pos(&mut self, focus: Vec3<f32>) { self.tgt_focus = focus; }

    /// Get the aspect ratio of the camera.
    pub fn get_aspect_ratio(&self) -> f32 { self.aspect }

    /// Set the aspect ratio of the camera.
    pub fn set_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = if aspect.is_normal() { aspect } else { 1.0 };
    }

    /// Get the orientation of the camera.
    pub fn get_orientation(&self) -> Vec3<f32> { self.ori }

    /// Get the field of view of the camera in radians.
    pub fn get_fov(&self) -> f32 { self.fov }

    /// Set the field of view of the camera in radians.
    pub fn set_fov(&mut self, fov: f32) { self.tgt_fov = fov; }

    /// Set the FOV in degrees
    pub fn set_fov_deg(&mut self, fov: u16) {
        //Magic value comes from pi/180; no use recalculating.
        self.set_fov((fov as f32) * 0.01745329)
    }

    /// Set the mode of the camera.
    pub fn set_mode(&mut self, mode: CameraMode) {
        if self.mode != mode {
            self.mode = mode;
            match self.mode {
                CameraMode::ThirdPerson => {
                    self.zoom_by(5.0);
                },
                CameraMode::FirstPerson => {
                    self.set_distance(MIN_ZOOM);
                },
                CameraMode::Freefly => {
                    self.zoom_by(0.0);
                },
            }
        }
    }

    /// Get the mode of the camera
    pub fn get_mode(&self) -> CameraMode {
        // Perfom a bit of a trick... don't report first-person until the camera has
        // lerped close enough to the player.
        match self.mode {
            CameraMode::FirstPerson if self.dist < 0.5 => CameraMode::FirstPerson,
            CameraMode::FirstPerson => CameraMode::ThirdPerson,
            mode => mode,
        }
    }

    /// Cycle the camera to its next valid mode. If is_admin is false then only
    /// modes which are accessible without admin access will be cycled to.
    pub fn next_mode(&mut self, is_admin: bool) {
        self.set_mode(match self.mode {
            CameraMode::ThirdPerson => CameraMode::FirstPerson,
            CameraMode::FirstPerson => {
                if is_admin {
                    CameraMode::Freefly
                } else {
                    CameraMode::ThirdPerson
                }
            },
            CameraMode::Freefly => CameraMode::ThirdPerson,
        });
    }

    /// Return a unit vector in the forward direction for the current camera
    /// orientation
    pub fn forward(&self) -> Vec3<f32> {
        Vec3::new(
            f32::sin(self.ori.x) * f32::cos(self.ori.y),
            f32::cos(self.ori.x) * f32::cos(self.ori.y),
            -f32::sin(self.ori.y),
        )
    }

    /// Return a unit vector in the right direction for the current camera
    /// orientation
    pub fn right(&self) -> Vec3<f32> {
        const UP: Vec3<f32> = Vec3::new(0.0, 0.0, 1.0);
        self.forward().cross(UP).normalized()
    }

    /// Return a unit vector in the forward direction on the XY plane for
    /// the current camera orientation
    pub fn forward_xy(&self) -> Vec2<f32> { Vec2::new(f32::sin(self.ori.x), f32::cos(self.ori.x)) }

    /// Return a unit vector in the right direction on the XY plane for
    /// the current camera orientation
    pub fn right_xy(&self) -> Vec2<f32> { Vec2::new(f32::cos(self.ori.x), -f32::sin(self.ori.x)) }
}
