//! Shared orbital camera for the 3D examples.

use bevy::prelude::*;

/// A camera orbiting the origin, driven by the arrow keys or WASD.
#[derive(Component)]
pub(crate) struct Orbit {
    pub(crate) yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) radius: f32,
}

impl Orbit {
    /// An orbit at `radius`, looking slightly down at the origin.
    pub(crate) fn new(radius: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
            radius,
        }
    }
}

pub(crate) fn orbit_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Orbit, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = camera.single_mut() else {
        return;
    };
    let speed = 1.5 * time.delta_secs();
    if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        orbit.yaw -= speed;
    }
    if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        orbit.yaw += speed;
    }
    if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        orbit.pitch += speed;
    }
    if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        orbit.pitch -= speed;
    }
    // Clamp short of the poles so `looking_at` doesn't degenerate when the view aligns with up.
    orbit.pitch = orbit.pitch.clamp(-1.5, 1.5);

    let pos = Vec3::new(
        orbit.radius * orbit.pitch.cos() * orbit.yaw.sin(),
        orbit.radius * orbit.pitch.sin(),
        orbit.radius * orbit.pitch.cos() * orbit.yaw.cos(),
    );
    *transform = Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y);
}
