//! Camera module for 3D scene rendering
//! 
//! Provides camera functionality including perspective/orthographic projection,
//! view matrix calculation, and position/direction management.

use cgmath::{Matrix4, Vector3, Point3, Rad, Zero, InnerSpace, EuclideanSpace, perspective, ortho};

/// Represents a 3D camera with configurable projection and positioning
#[derive(Debug, Clone)]
pub struct Camera3D {
    /// Viewport dimensions
    pub viewport_width: i32,
    pub viewport_height: i32,
    
    /// Projection mode
    pub is_orthographic: bool,
    pub orthographic_scale: f32,
    
    /// Perspective projection settings
    pub field_of_view: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    
    /// Camera position and orientation
    pub position: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub forward: Vector3<f32>,
    
    /// Mirror mode for left/right handed display
    pub mirror_left_right: bool,
}

impl Camera3D {
    /// Creates a new camera with default settings
    pub fn new() -> Self {
        Self {
            viewport_width: 1024,
            viewport_height: 768,
            is_orthographic: false,
            orthographic_scale: 1.0,
            field_of_view: std::f32::consts::FRAC_PI_4, // 45 degrees
            near_plane: 1.0,
            far_plane: 10000.0,
            position: Vector3::new(0.0, 0.0, 5.0),
            up: Vector3::unit_y(),
            right: Vector3::zero(),
            forward: Vector3::unit_z(),
            mirror_left_right: false,
        }
    }
    
    /// Sets the camera to look at a specific point
    pub fn set_look_at(&mut self, look_at: Vector3<f32>) {
        self.forward = (look_at - self.position).normalize();
        self.right = self.forward.cross(Vector3::unit_y()).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }
    
    /// Gets the projection matrix based on current settings
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        if self.is_orthographic {
            let width = self.viewport_width as f32 / self.orthographic_scale;
            let height = self.viewport_height as f32 / self.orthographic_scale;
            ortho(-width / 2.0, width / 2.0, -height / 2.0, height / 2.0, self.near_plane, self.far_plane)
        } else {
            let aspect = self.viewport_width as f32 / self.viewport_height as f32;
            perspective(Rad(self.field_of_view), aspect, self.near_plane, self.far_plane)
        }
    }
    
    /// Gets the view matrix for the camera
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let look_at = self.position + self.forward;
        
        if self.mirror_left_right {
            let view = Matrix4::look_at_rh(Point3::from_vec(self.position), Point3::from_vec(look_at), self.up);
            view * Matrix4::new(
                -1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            )
        } else {
            Matrix4::look_at_rh(Point3::from_vec(self.position), Point3::from_vec(look_at), self.up)
        }
    }
    
    /// Calculates the distance needed for a given width at the current FOV
    pub fn get_distance_for_width(&self, width: f32) -> f32 {
        width / (2.0 * (self.field_of_view / 2.0).tan())
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera specialized for guitar fretboard viewing
#[derive(Debug, Clone)]
pub struct FretCamera {
    /// Base camera settings
    pub base: Camera3D,
    
    /// Fret camera specific settings
    pub camera_distance: f32,
    pub focus_dist: f32,
    target_camera_distance: f32,
    position_fret: f32,
}

impl FretCamera {
    /// Creates a new fret camera
    pub fn new() -> Self {
        let mut base = Camera3D::new();
        base.position = Vector3::new(0.0, 0.0, 5.0);
        base.up = Vector3::unit_y();
        base.forward = Vector3::new(0.0, 0.0, -1.0);
        base.field_of_view = std::f32::consts::FRAC_PI_4;
        
        Self {
            base,
            camera_distance: 70.0,
            focus_dist: 600.0,
            target_camera_distance: 75.0,
            position_fret: 3.0,
        }
    }
    
    /// Updates the camera position based on fret range
    pub fn update(&mut self, min_fret: f32, max_fret: f32, target_focus_fret: f32, focus_y: f32) {
        if min_fret <= max_fret {
            let fret_dist = max_fret - min_fret;
            let adjusted_dist = (fret_dist - 12.0).max(0.0);
            self.target_camera_distance = 65.0 + (adjusted_dist.max(4.0) * 3.0);
            
            let mut target_position_fret = (max_fret + min_fret) / 2.0;
            
            if target_position_fret < (target_focus_fret - 3.0) {
                target_position_fret = target_focus_fret - 3.0;
            }
            if target_position_fret > (target_focus_fret + 5.0) {
                target_position_fret = target_focus_fret + 5.0;
            }
            
            self.position_fret = lerp(self.position_fret, clamp(target_position_fret, 3.5, 24.0) - 1.0, 0.02);
        }
        
        self.camera_distance = lerp(self.camera_distance, self.target_camera_distance, 0.01);
        
        let fret_offset = (10.0 - self.position_fret) / 4.0;
        self.base.position = Vector3::new(get_fret_position(self.position_fret + fret_offset), 50.0, focus_y + self.camera_distance);
        
        let look_target = Vector3::new(get_fret_position(self.position_fret), 0.0, self.base.position.z - (self.focus_dist * 0.3));
        self.base.set_look_at(look_target);
    }
}

impl Default for FretCamera {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates the position of a fret on the guitar neck
pub fn get_fret_position(fret: f32) -> f32 {
    const SCALE_LENGTH: f32 = 300.0;
    SCALE_LENGTH - (SCALE_LENGTH / (2.0_f32.powf(fret / 12.0)))
}

/// Linear interpolation helper
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp helper
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}