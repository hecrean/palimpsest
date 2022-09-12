use bevy::prelude::*;
use std::ops::RangeInclusive;
use bevy::{render::camera::Camera, input::mouse::{MouseMotion, MouseWheel, MouseScrollUnit::{Line, Pixel}}};

const LINE_TO_PIXEL_RATIO: f32 = 0.1;


pub enum CameraEvents {
    Pan(Vec2),
    Orbit(Vec2),
    Zoom(f32)
}

#[derive(Component)]
pub struct OrbitCamera {
    origin: Vec3,
    θ: f32, // polar
    ϕ: f32, // azimuthal
    ρ: f32, // radial
    θ_range: RangeInclusive<f32>,
    ϕ_range: RangeInclusive<f32>,
    ρ_range: RangeInclusive<f32>,
    rotate_sensitivity: f32, 
    pan_sensitivity: f32, 
    zoom_sensitivity: f32, 
    rotate_button: MouseButton,
    pan_button: MouseButton,
    enabled: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            θ: 0.0,
            ϕ: std::f32::consts::FRAC_PI_2,
            ρ: 5.0,
            θ_range: 0.01..= std::f32::consts::PI,
            ϕ_range:0.01..= std::f32::consts::FRAC_PI_2,
            ρ_range: 0.01..= 1000.,
            origin: Vec3::ZERO,
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            rotate_button: MouseButton::Left,
            pan_button: MouseButton::Right,
            enabled: true,
        }
    }
}

impl OrbitCamera {
    pub fn new(ρ: f32, origin: Vec3) -> OrbitCamera {
        OrbitCamera {
            ρ,
            origin,
            ..Self::default()
        }
    }
}

pub struct OrbitCameraPlugin;

impl OrbitCameraPlugin {
    fn update_transform_system( mut query: Query<(&OrbitCamera, &mut Transform), (With<Camera>, Changed<OrbitCamera>)>) -> (){
        for (orbit_camera, mut transform) in query.iter_mut() {
            if orbit_camera.enabled {
                let rot = Quat::from_axis_angle(Vec3::Y, orbit_camera.θ)
                    * Quat::from_axis_angle(-Vec3::X, orbit_camera.ϕ);
                transform.translation = (rot * Vec3::Y) * orbit_camera.ρ + orbit_camera.origin;
                transform.look_at(orbit_camera.origin, Vec3::Y);
            }
        }
    }
    pub fn emit_motion_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_motion_events: EventReader<MouseMotion>,
        mouse_button_input: Res<Input<MouseButton>>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut mouse_position_delta = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            mouse_position_delta += event.delta;
        }
        for orbit_camera in query.iter_mut() {
            if orbit_camera.enabled {
                if mouse_button_input.pressed(orbit_camera.rotate_button) {
                    events.send(CameraEvents::Orbit(mouse_position_delta))
                }

                if mouse_button_input.pressed(orbit_camera.pan_button) {
                    events.send(CameraEvents::Pan(mouse_position_delta))
                }
            }
        }
    }

    pub fn mouse_motion_system(
        time: Res<Time>,
        mut events: EventReader<CameraEvents>,
        mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ) {
        for (mut camera, transform, _) in query.iter_mut() {
            if !camera.enabled {
                continue;
            }

            for event in events.iter() {
                match event {
                    CameraEvents::Orbit(delta) => {
                        camera.θ -= delta.x * camera.rotate_sensitivity * time.delta_seconds();
                        camera.ϕ -= delta.y * camera.rotate_sensitivity * time.delta_seconds();
                        camera.ϕ = camera
                            .ϕ
                            .max(*camera.ϕ_range.start())
                            .min(*camera.ϕ_range.end());
                    }
                    CameraEvents::Pan(delta) => {
                        let right_dir = transform.rotation * -Vec3::X;
                        let up_dir = transform.rotation * Vec3::Y;
                        let pan_vector = (delta.x * right_dir + delta.y * up_dir)
                            * camera.pan_sensitivity
                            * time.delta_seconds();
                        camera.origin += pan_vector;
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn emit_zoom_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_wheel_events: EventReader<MouseWheel>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut total = 0.0;
        for event in mouse_wheel_events.iter() {
            total += event.y
                * match event.unit {
                    Line => 1.0,
                    Pixel => LINE_TO_PIXEL_RATIO,
                };
        }

        if total != 0.0 {
            for camera in query.iter_mut() {
                if camera.enabled {
                    events.send(CameraEvents::Zoom(total));
                }
            }
        }
    }

    pub fn zoom_system(
        mut query: Query<&mut OrbitCamera, With<Camera>>,
        mut events: EventReader<CameraEvents>,
    ) {
        for mut camera in query.iter_mut() {
            for event in events.iter() {
                if camera.enabled {
                    if let CameraEvents::Zoom(distance) = event {
                        camera.ρ *= camera.zoom_sensitivity.powf(*distance);
                    }
                }
            }
        }
    }
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::emit_motion_events)
            .add_system(Self::mouse_motion_system)
            .add_system(Self::emit_zoom_events)
            .add_system(Self::zoom_system)
            .add_system(Self::update_transform_system)
            .add_event::<CameraEvents>();
    }
}