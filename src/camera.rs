use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(8.0, 40.0, 8.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let direction = self.get_direction();
        let right = direction.cross(Vec3::Y).normalize();
        let up = right.cross(direction).normalize();

        Mat4::look_at_rh(self.position, self.position + direction, up)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn get_direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn get_forward(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_forward().cross(Vec3::Y).normalize()
    }

    pub fn update_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
