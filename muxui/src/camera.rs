//! # Standard 3D Camera Controllers
//!
//! Provides pre-built, ergonomic camera controllers (`FlyCamera`, `OrbitCamera`, and `ChaseCamera`)
//! that eliminate repetitive Euler trigonometry, basis vector calculation, and pitch clamping boilerplate.

use muxutils::raylib::*;
use crate::d3::Camera3DBuilder;

/// First-person / free-fly 3D camera controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlyCamera {
    /// Camera position in 3D world space.
    pub position: Vector3,
    /// Horizontal yaw angle in degrees.
    pub yaw: f32,
    /// Vertical pitch angle in degrees (clamped to `[-85.0, 85.0]`).
    pub pitch: f32,
    /// Base movement speed in world units per second.
    pub speed: f32,
    /// Rotation sensitivity multiplier.
    pub sensitivity: f32,
    /// Vertical field of view in degrees.
    pub fovy: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self::new(Vector3::new(0.0, 10.0, 10.0))
    }
}

impl FlyCamera {
    /// Creates a new [`FlyCamera`] at the specified world position looking toward `-Z`.
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            yaw: 180.0,
            pitch: -15.0,
            speed: 20.0,
            sensitivity: 60.0,
            fovy: 60.0,
        }
    }

    /// Builder method to set initial yaw and pitch angles (in degrees).
    pub fn with_yaw_pitch(mut self, yaw: f32, pitch: f32) -> Self {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-85.0, 85.0);
        self
    }

    /// Builder method to set the base movement speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Builder method to set the rotation sensitivity.
    pub fn with_sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    /// Builder method to set the vertical field of view in degrees.
    pub fn with_fovy(mut self, fovy: f32) -> Self {
        self.fovy = fovy;
        self
    }

    /// Computes the normalized 3D forward direction vector from the current Euler `yaw` and `pitch`.
    pub fn forward(&self) -> Vector3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        Vector3::new(
            pitch_rad.cos() * yaw_rad.sin(),
            pitch_rad.sin(),
            pitch_rad.cos() * yaw_rad.cos(),
        )
        .normalize()
    }

    /// Computes the normalized horizontal right vector on the XZ plane.
    pub fn right(&self) -> Vector3 {
        let yaw_rad = self.yaw.to_radians();
        Vector3::new(-yaw_rad.cos(), 0.0, yaw_rad.sin()).normalize()
    }

    /// Computes the camera's local orthogonal up vector.
    pub fn up(&self) -> Vector3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Rotates the camera by `yaw_delta` and `pitch_delta` (in degrees), clamping pitch to `[-85.0, 85.0]`.
    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw = (self.yaw + yaw_delta) % 360.0;
        self.pitch = (self.pitch + pitch_delta).clamp(-85.0, 85.0);
    }

    /// Translates the camera along its local forward, right, and world-up axes scaled by `self.speed * dt`.
    pub fn move_local(&mut self, forward_input: f32, right_input: f32, up_input: f32, dt: f32) {
        let fwd = self.forward();
        let rgt = self.right();
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let dir = fwd * forward_input + rgt * right_input + world_up * up_input;
        self.position = self.position + dir * (self.speed * dt);
    }

    /// Converts the controller state into a Raylib [`Camera3D`].
    pub fn to_camera_3d(&self) -> Camera3D {
        Camera3DBuilder::new()
            .position(self.position)
            .target(self.position + self.forward())
            .up(Vector3::new(0.0, 1.0, 0.0))
            .fovy(self.fovy)
            .build()
    }
}

/// Spherical orbital 3D camera controller revolving around a target point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCamera {
    /// Center point the camera orbits around.
    pub target: Vector3,
    /// Radial distance from `target`.
    pub distance: f32,
    /// Horizontal orbital angle in degrees.
    pub yaw: f32,
    /// Vertical elevation angle in degrees.
    pub pitch: f32,
    /// Minimum allowed pitch angle in degrees.
    pub min_pitch: f32,
    /// Maximum allowed pitch angle in degrees.
    pub max_pitch: f32,
    /// Minimum zoom distance.
    pub min_distance: f32,
    /// Maximum zoom distance.
    pub max_distance: f32,
    /// Vertical field of view in degrees.
    pub fovy: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::new(Vector3::zero(), 14.0)
    }
}

impl OrbitCamera {
    /// Creates a new [`OrbitCamera`] orbiting `target` at the specified `distance`.
    pub fn new(target: Vector3, distance: f32) -> Self {
        Self {
            target,
            distance: distance.max(0.1),
            yaw: 45.0,
            pitch: 30.0,
            min_pitch: -80.0,
            max_pitch: 85.0,
            min_distance: 1.0,
            max_distance: 500.0,
            fovy: 45.0,
        }
    }

    /// Builder method to set initial orbital `yaw` and `pitch` angles in degrees.
    pub fn with_angles(mut self, yaw: f32, pitch: f32) -> Self {
        self.yaw = yaw;
        self.pitch = pitch.clamp(self.min_pitch, self.max_pitch);
        self
    }

    /// Builder method to set the vertical pitch limits in degrees.
    pub fn with_pitch_limits(mut self, min_pitch: f32, max_pitch: f32) -> Self {
        self.min_pitch = min_pitch;
        self.max_pitch = max_pitch;
        self.pitch = self.pitch.clamp(self.min_pitch, self.max_pitch);
        self
    }

    /// Builder method to set the vertical field of view in degrees.
    pub fn with_fovy(mut self, fovy: f32) -> Self {
        self.fovy = fovy;
        self
    }

    /// Rotates the orbital angles by `yaw_delta` and `pitch_delta` (in degrees).
    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw = (self.yaw + yaw_delta) % 360.0;
        self.pitch = (self.pitch + pitch_delta).clamp(self.min_pitch, self.max_pitch);
    }

    /// Adjusts the orbital distance by `distance_delta`, clamped to `[min_distance, max_distance]`.
    pub fn zoom(&mut self, distance_delta: f32) {
        self.distance = (self.distance + distance_delta).clamp(self.min_distance, self.max_distance);
    }

    /// Computes the camera's 3D world position on the orbital sphere.
    pub fn position(&self) -> Vector3 {
        let rad_yaw = self.yaw.to_radians();
        let rad_pitch = self.pitch.to_radians();
        let offset = Vector3::new(
            self.distance * rad_pitch.cos() * rad_yaw.sin(),
            self.distance * rad_pitch.sin(),
            self.distance * rad_pitch.cos() * rad_yaw.cos(),
        );
        self.target + offset
    }

    /// Converts the controller state into a Raylib [`Camera3D`].
    pub fn to_camera_3d(&self) -> Camera3D {
        Camera3DBuilder::new()
            .position(self.position())
            .target(self.target)
            .up(Vector3::new(0.0, 1.0, 0.0))
            .fovy(self.fovy)
            .build()
    }
}

/// Third-person spring-smoothed chase camera controller for vehicles, aircraft, and characters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChaseCamera {
    /// Current smoothed camera world position.
    pub position: Vector3,
    /// Current look-at target world position.
    pub target: Vector3,
    /// Current camera up vector.
    pub up: Vector3,
    /// Trailing distance behind the target along its negative forward vector.
    pub distance: f32,
    /// Vertical offset above the target along its up vector.
    pub height_offset: f32,
    /// Forward look-ahead distance ahead of the target.
    pub look_ahead: f32,
    /// Interpolation responsiveness / spring stiffness.
    pub stiffness: f32,
    /// Vertical field of view in degrees.
    pub fovy: f32,
}

impl Default for ChaseCamera {
    fn default() -> Self {
        Self::new(16.0, 4.5)
    }
}

impl ChaseCamera {
    /// Creates a new [`ChaseCamera`] with the given trailing `distance` and `height_offset`.
    pub fn new(distance: f32, height_offset: f32) -> Self {
        Self {
            position: Vector3::new(0.0, height_offset, -distance),
            target: Vector3::zero(),
            up: Vector3::new(0.0, 1.0, 0.0),
            distance,
            height_offset,
            look_ahead: 15.0,
            stiffness: 9.0,
            fovy: 70.0,
        }
    }

    /// Builder method to set the spring stiffness and look-ahead distance.
    pub fn with_smoothing(mut self, stiffness: f32, look_ahead: f32) -> Self {
        self.stiffness = stiffness;
        self.look_ahead = look_ahead;
        self
    }

    /// Builder method to set the vertical field of view in degrees.
    pub fn with_fovy(mut self, fovy: f32) -> Self {
        self.fovy = fovy;
        self
    }

    /// Updates the chase camera to smoothly track an object given its world position and basis vectors.
    pub fn follow(&mut self, target_pos: Vector3, target_forward: Vector3, target_up: Vector3, dt: f32) {
        let desired_pos = target_pos - target_forward * self.distance + target_up * self.height_offset;
        let blend = (dt * self.stiffness).clamp(0.0, 1.0);
        self.position = self.position.lerp(desired_pos, blend);
        self.target = target_pos + target_forward * self.look_ahead;
        self.up = self.up.lerp(target_up, blend).normalize();
    }

    /// Converts the controller state into a Raylib [`Camera3D`].
    pub fn to_camera_3d(&self) -> Camera3D {
        Camera3DBuilder::new()
            .position(self.position)
            .target(self.target)
            .up(self.up)
            .fovy(self.fovy)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fly_camera_navigation() {
        let mut cam = FlyCamera::new(Vector3::zero())
            .with_yaw_pitch(0.0, 0.0)
            .with_speed(10.0)
            .with_fovy(75.0);

        // At yaw=0, pitch=0, forward is (0, 0, 1)
        let fwd = cam.forward();
        assert!((fwd.x - 0.0).abs() < 1e-5);
        assert!((fwd.y - 0.0).abs() < 1e-5);
        assert!((fwd.z - 1.0).abs() < 1e-5);

        // Move forward for 1.0s at speed 10.0 -> z = 10.0
        cam.move_local(1.0, 0.0, 0.0, 1.0);
        assert!((cam.position.z - 10.0).abs() < 1e-4);

        // Pitch clamping
        cam.rotate(90.0, 200.0);
        assert_eq!(cam.pitch, 85.0);
        assert_eq!(cam.yaw, 90.0);

        let rl_cam = cam.to_camera_3d();
        assert_eq!(rl_cam.fovy, 75.0);
    }

    #[test]
    fn test_orbit_camera_positioning() {
        let mut orbit = OrbitCamera::new(Vector3::zero(), 10.0)
            .with_angles(0.0, 0.0)
            .with_pitch_limits(-45.0, 45.0);

        // At yaw=0, pitch=0, position is (0, 0, 10)
        let pos = orbit.position();
        assert!((pos.x - 0.0).abs() < 1e-5);
        assert!((pos.y - 0.0).abs() < 1e-5);
        assert!((pos.z - 10.0).abs() < 1e-5);

        orbit.zoom(-5.0);
        assert_eq!(orbit.distance, 5.0);

        orbit.rotate(0.0, 60.0);
        assert_eq!(orbit.pitch, 45.0);
    }

    #[test]
    fn test_chase_camera_follow() {
        let mut chase = ChaseCamera::new(10.0, 5.0).with_smoothing(10.0, 20.0);
        chase.follow(
            Vector3::new(0.0, 10.0, 50.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
            1.0,
        );
        assert_eq!(chase.position, Vector3::new(0.0, 15.0, 40.0));
        assert_eq!(chase.target, Vector3::new(0.0, 10.0, 70.0));
    }
}
