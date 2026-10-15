//! # 3D Rendering and Camera Abstractions
//!
//! Provides safe RAII wrappers around Raylib's 3D drawing mode, OpenGL matrix operations,
//! and camera management.

use muxutils::raylib::*;

/// Fluent builder for constructing a [`Camera3D`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3DBuilder {
    camera: Camera3D,
}

impl Default for Camera3DBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera3DBuilder {
    /// Creates a new `Camera3DBuilder` with sensible defaults.
    pub fn new() -> Self {
        Self {
            camera: Camera3D {
                position: Vector3::new(0.0, 10.0, 10.0),
                target: Vector3::zero(),
                up: Vector3::new(0.0, 1.0, 0.0),
                fovy: 45.0,
                projection: CameraProjection::CAMERA_PERSPECTIVE as i32,
            },
        }
    }

    /// Sets the camera position in 3D world space.
    pub fn position(mut self, pos: Vector3) -> Self {
        self.camera.position = pos;
        self
    }

    /// Sets the target point the camera is looking at.
    pub fn target(mut self, target: Vector3) -> Self {
        self.camera.target = target;
        self
    }

    /// Sets the camera's up vector (e.g. `(0.0, 1.0, 0.0)` for Y-up).
    pub fn up(mut self, up: Vector3) -> Self {
        self.camera.up = up;
        self
    }

    /// Sets the camera field of view in the Y axis in degrees (or ortho size).
    pub fn fovy(mut self, fovy: f32) -> Self {
        self.camera.fovy = fovy;
        self
    }

    /// Sets the camera projection type (e.g. [`CameraProjection::CAMERA_PERSPECTIVE`] or [`CameraProjection::CAMERA_ORTHOGRAPHIC`]).
    pub fn projection(mut self, projection: CameraProjection) -> Self {
        self.camera.projection = projection as i32;
        self
    }

    /// Builds and returns the configured [`Camera3D`].
    pub fn build(self) -> Camera3D {
        self.camera
    }
}

/// Safe drawing context active inside a [`draw_3d`] closure.
pub struct Render3DContext<'a> {
    pub(crate) _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Render3DContext<'a> {
    /// Executes a closure with an isolated OpenGL matrix transformation stack.
    ///
    /// Automatically pushes a new matrix via `rlPushMatrix` and restores it via `rlPopMatrix`
    /// when the closure terminates, ensuring matrix state is preserved even in early returns or panics.
    pub fn push_matrix<F: FnOnce(&mut Render3DContext)>(&mut self, f: F) {
        struct MatrixGuard;
        impl Drop for MatrixGuard {
            fn drop(&mut self) {
                unsafe {
                    rlgl::rlPopMatrix();
                }
            }
        }

        unsafe {
            rlgl::rlPushMatrix();
        }
        let _guard = MatrixGuard;
        f(self);
    }

    /// Rotates the current coordinate system by `angle_deg` degrees around `axis`.
    pub fn rotate(&mut self, angle_deg: f32, axis: Vector3) {
        unsafe {
            rlgl::rlRotatef(angle_deg, axis.x, axis.y, axis.z);
        }
    }

    /// Translates the current coordinate system by `translation`.
    pub fn translate(&mut self, translation: Vector3) {
        unsafe {
            rlgl::rlTranslatef(translation.x, translation.y, translation.z);
        }
    }

    /// Scales the current coordinate system by `scale`.
    pub fn scale(&mut self, scale: Vector3) {
        unsafe {
            rlgl::rlScalef(scale.x, scale.y, scale.z);
        }
    }

    /// Draws a solid 3D cube.
    pub fn draw_cube(&self, position: Vector3, width: f32, height: f32, length: f32, color: Color) {
        unsafe {
            DrawCube(position, width, height, length, color);
        }
    }

    /// Draws a wireframe 3D cube.
    pub fn draw_cube_wires(&self, position: Vector3, width: f32, height: f32, length: f32, color: Color) {
        unsafe {
            DrawCubeWires(position, width, height, length, color);
        }
    }

    /// Draws a solid 3D sphere.
    pub fn draw_sphere(&self, center: Vector3, radius: f32, color: Color) {
        unsafe {
            DrawSphere(center, radius, color);
        }
    }

    /// Draws a wireframe 3D sphere.
    pub fn draw_sphere_wires(&self, center: Vector3, radius: f32, rings: i32, slices: i32, color: Color) {
        unsafe {
            DrawSphereWires(center, radius, rings, slices, color);
        }
    }

    /// Draws a 3D cylinder.
    pub fn draw_cylinder(
        &self,
        position: Vector3,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        sides: i32,
        color: Color,
    ) {
        unsafe {
            DrawCylinder(position, radius_top, radius_bottom, height, sides, color);
        }
    }

    /// Draws a wireframe 3D cylinder.
    pub fn draw_cylinder_wires(
        &self,
        position: Vector3,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        sides: i32,
        color: Color,
    ) {
        unsafe {
            DrawCylinderWires(position, radius_top, radius_bottom, height, sides, color);
        }
    }

    /// Draws a 3D ground plane.
    pub fn draw_plane(&self, center: Vector3, size: Vector2, color: Color) {
        unsafe {
            DrawPlane(center, size, color);
        }
    }

    /// Draws a 3D line between `start` and `end`.
    pub fn draw_line_3d(&self, start: Vector3, end: Vector3, color: Color) {
        unsafe {
            DrawLine3D(start, end, color);
        }
    }

    /// Draws a 3D grid centered at origin.
    pub fn draw_grid(&self, slices: i32, spacing: f32) {
        unsafe {
            DrawGrid(slices, spacing);
        }
    }

    /// Draws a triangle mesh safely chunking draw calls to prevent `rlgl` vertex buffer overflows.
    ///
    /// Raylib's internal batch buffer defaults to 8,192 vertices. Pushing large procedural meshes
    /// without chunking and checking batch limits causes vertices to be dropped silently.
    /// This method automatically batches vertices into safe slices divisible by 3 (4,095 vertices max per chunk).
    pub fn draw_trimesh_buffered(&self, vertices: &[Vertex3D]) {
        const MAX_CHUNK_VERTICES: usize = 4095; // divisible by 3 (1365 triangles)
        for chunk in vertices.chunks(MAX_CHUNK_VERTICES) {
            unsafe {
                rlgl::rlCheckRenderBatchLimit(chunk.len() as i32);
                rlgl::rlBegin(rlgl::RL_TRIANGLES as i32);
                for v in chunk {
                    rlgl::rlColor4ub(v.color.r, v.color.g, v.color.b, v.color.a);
                    if let Some(norm) = v.normal {
                        rlgl::rlNormal3f(norm.x, norm.y, norm.z);
                    }
                    if let Some(uv) = v.uv {
                        rlgl::rlTexCoord2f(uv.x, uv.y);
                    }
                    rlgl::rlVertex3f(v.position.x, v.position.y, v.position.z);
                }
                rlgl::rlEnd();
            }
        }
    }

    /// Convenience method to draw flat colored triangles from position and color slices.
    pub fn draw_triangles(&self, positions: &[Vector3], colors: &[Color]) {
        let count = positions.len().min(colors.len());
        let vertices: Vec<Vertex3D> = (0..count)
            .map(|i| Vertex3D::new(positions[i], colors[i]))
            .collect();
        self.draw_trimesh_buffered(&vertices);
    }

    /// Draws a 3D billboard texture facing the camera.
    pub fn draw_billboard(&self, texture: Texture2D, position: Vector3, size: f32, tint: Color, camera: &Camera3D) {
        draw_billboard(*camera, texture, position, size, tint);
    }
}

/// A 3D vertex with position, color, and optional normal and texture coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    /// 3D position vector.
    pub position: Vector3,
    /// Vertex color.
    pub color: Color,
    /// Optional surface normal vector.
    pub normal: Option<Vector3>,
    /// Optional UV texture coordinates.
    pub uv: Option<Vector2>,
}

impl Vertex3D {
    /// Creates a new vertex with position and color.
    pub fn new(position: Vector3, color: Color) -> Self {
        Self {
            position,
            color,
            normal: None,
            uv: None,
        }
    }

    /// Sets the normal vector for this vertex.
    pub fn with_normal(mut self, normal: Vector3) -> Self {
        self.normal = Some(normal);
        self
    }

    /// Sets the UV texture coordinates for this vertex.
    pub fn with_uv(mut self, uv: Vector2) -> Self {
        self.uv = Some(uv);
        self
    }
}

/// Executes a 3D rendering pass with the specified [`Camera3D`].
///
/// Sets up the 3D projection mode and guarantees cleanup via an RAII drop guard,
/// eliminating manual `unsafe { BeginMode3D(...) }` and `EndMode3D()` boilerplate.
pub fn draw_3d<F: FnOnce(&mut Render3DContext)>(camera: Camera3D, f: F) {
    struct Mode3DGuard;
    impl Drop for Mode3DGuard {
        fn drop(&mut self) {
            unsafe {
                EndMode3D();
            }
        }
    }

    unsafe {
        BeginMode3D(camera);
    }
    let _guard = Mode3DGuard;
    let mut ctx = Render3DContext {
        _marker: std::marker::PhantomData,
    };
    f(&mut ctx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_builder_defaults_and_customization() {
        let default_cam = Camera3DBuilder::new().build();
        assert_eq!(default_cam.position, Vector3::new(0.0, 10.0, 10.0));
        assert_eq!(default_cam.target, Vector3::zero());
        assert_eq!(default_cam.up, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(default_cam.fovy, 45.0);
        assert_eq!(default_cam.projection, CameraProjection::CAMERA_PERSPECTIVE as i32);

        let custom_cam = Camera3DBuilder::new()
            .position(Vector3::new(5.0, 15.0, -20.0))
            .target(Vector3::new(1.0, 2.0, 3.0))
            .up(Vector3::new(0.0, 0.0, 1.0))
            .fovy(75.0)
            .projection(CameraProjection::CAMERA_ORTHOGRAPHIC)
            .build();

        assert_eq!(custom_cam.position, Vector3::new(5.0, 15.0, -20.0));
        assert_eq!(custom_cam.target, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(custom_cam.up, Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(custom_cam.fovy, 75.0);
        assert_eq!(custom_cam.projection, CameraProjection::CAMERA_ORTHOGRAPHIC as i32);
    }

    #[test]
    fn test_vertex_3d_builder() {
        let v = Vertex3D::new(Vector3::new(1.0, 2.0, 3.0), RED)
            .with_normal(Vector3::new(0.0, 1.0, 0.0))
            .with_uv(Vector2::new(0.5, 0.5));

        assert_eq!(v.position, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(v.color, RED);
        assert_eq!(v.normal, Some(Vector3::new(0.0, 1.0, 0.0)));
        assert_eq!(v.uv, Some(Vector2::new(0.5, 0.5)));
    }
}
