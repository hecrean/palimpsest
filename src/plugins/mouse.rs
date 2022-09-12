use bevy::{ecs::system::Resource, input::mouse, prelude::Vec2, prelude::*, reflect::Reflect};

#[derive(Resource, Reflect, Debug, Clone)]
pub struct Mouse {
    // button: MouseButton,
    // button_state: ButtonState,
    pub normalised_device_coordinates: Vec2,
}
impl Default for Mouse {
    fn default() -> Mouse {
        Mouse {
            normalised_device_coordinates: Vec2::new(0., 0.),
        }
    }
}

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mouse>()
            .add_system(mouse_position_system);
    }
}

fn mouse_position_system(windows: Res<Windows>, mut mouse: ResMut<Mouse>) {
    // Games typically only have one window (the primary window).
    // For multi-window applications, you need to use a specific window ID here.
    let window = windows.get_primary().unwrap();

    if let Some(_position) = window.cursor_position() {
        // cursor is inside the window, position given
        let screen_pos = window.cursor_position().unwrap();
        let window_size = Vec2::new(window.width(), window.height());
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        mouse.normalised_device_coordinates = ndc;
    } else {
        // cursor is not inside the window
    }
}
