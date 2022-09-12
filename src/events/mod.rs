use bevy::{
    ecs::{event::EventReader, system::ResMut},
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        touch::{ForceTouch, TouchInput, TouchPhase},
        ButtonState,
    },
    math::Vec2,
    utils::HashMap,
};

fn mouse_button_events(mut mousebtn_evr: EventReader<MouseButtonInput>) {
    for ev in mousebtn_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                println!("Mouse button press: {:?}", ev.button);
            }
            ButtonState::Released => {
                println!("Mouse button release: {:?}", ev.button);
            }
        }
    }
}

fn mouse_move_events(mut mouse_move_evr: EventReader<MouseMotion>) {
    for ev in mouse_move_evr.iter() {
        println!("mouse moved {}", ev.delta);
    }
}

fn mouse_wheel_events(mut mouse_wheel_evr: EventReader<MouseWheel>) {
    for ev in mouse_wheel_evr.iter() {}
}

fn touch_events(mut touch_evr: EventReader<TouchInput>) {
    for ev in touch_evr.iter() {
        // in real apps you probably want to store and track touch ids somewhere
        match ev.phase {
            TouchPhase::Started => {
                println!("Touch {} started at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Moved => {
                println!("Touch {} moved to: {:?}", ev.id, ev.position);
            }
            TouchPhase::Ended => {
                println!("Touch {} ended at: {:?}", ev.id, ev.position);
            }
            TouchPhase::Cancelled => {
                println!("Touch {} cancelled at: {:?}", ev.id, ev.position);
            }
        }
    }
}

/// A touch input.
///
/// ## Usage
///
/// It is used to store the position and force of a touch input and also the `id` of the finger.
/// The data of the touch input comes from the [`TouchInput`] event and is being stored
/// inside of the [`Touches`] `bevy` resource.
#[derive(Debug, Clone, Copy)]
pub struct Touch {
    /// The id of the touch input.
    id: u64,
    /// The starting position of the touch input.
    start_position: Vec2,
    /// The starting force of the touch input.
    start_force: Option<ForceTouch>,
    /// The previous position of the touch input.
    previous_position: Vec2,
    /// The previous force of the touch input.
    previous_force: Option<ForceTouch>,
    /// The current position of the touch input.
    position: Vec2,
    /// The current force of the touch input.
    force: Option<ForceTouch>,
}

impl Touch {
    /// The delta of the current `position` and the `previous_position`.
    pub fn delta(&self) -> Vec2 {
        self.position - self.previous_position
    }

    /// The distance of the `start_position` and the current `position`.
    pub fn distance(&self) -> Vec2 {
        self.position - self.start_position
    }

    /// Returns the `id` of the touch.
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the `start_position` of the touch.
    #[inline]
    pub fn start_position(&self) -> Vec2 {
        self.start_position
    }

    /// Returns the `start_force` of the touch.
    #[inline]
    pub fn start_force(&self) -> Option<ForceTouch> {
        self.start_force
    }

    /// Returns the `previous_position` of the touch.
    #[inline]
    pub fn previous_position(&self) -> Vec2 {
        self.previous_position
    }

    /// Returns the `previous_force` of the touch.
    #[inline]
    pub fn previous_force(&self) -> Option<ForceTouch> {
        self.previous_force
    }

    /// Returns the current `position` of the touch.
    #[inline]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the current `force` of the touch.
    #[inline]
    pub fn force(&self) -> Option<ForceTouch> {
        self.force
    }
}

impl From<&TouchInput> for Touch {
    fn from(input: &TouchInput) -> Touch {
        Touch {
            id: input.id,
            start_position: input.position,
            start_force: input.force,
            previous_position: input.position,
            previous_force: input.force,
            position: input.position,
            force: input.force,
        }
    }
}

/// A collection of [`Touch`]es.
///
/// ## Usage
///
/// It is used to create a `bevy` resource that stores the data of the touches on a touchscreen
/// and can be accessed inside of a system.
///
/// ## Updating
///
/// The resource is updated inside of the [`touch_screen_input_system`](crate::touch::touch_screen_input_system).
#[derive(Debug, Clone, Default)]
pub struct Touches {
    /// A collection of every [`Touch`] that is currently being pressed.
    pressed: HashMap<u64, Touch>,
    /// A collection of every [`Touch`] that just got pressed.
    just_pressed: HashMap<u64, Touch>,
    /// A collection of every [`Touch`] that just got released.
    just_released: HashMap<u64, Touch>,
    /// A collection of every [`Touch`] that just got cancelled.
    just_cancelled: HashMap<u64, Touch>,
}

impl Touches {
    /// An iterator visiting every pressed [`Touch`] input in arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = &Touch> + '_ {
        self.pressed.values()
    }

    /// Returns the [`Touch`] input corresponding to the `id` if it is being pressed.
    pub fn get_pressed(&self, id: u64) -> Option<&Touch> {
        self.pressed.get(&id)
    }

    /// Returns `true` if the input corresponding to the `id` has just been pressed.
    pub fn just_pressed(&self, id: u64) -> bool {
        self.just_pressed.contains_key(&id)
    }

    /// An iterator visiting every just pressed [`Touch`] input in arbitrary order.
    pub fn iter_just_pressed(&self) -> impl Iterator<Item = &Touch> {
        self.just_pressed.values()
    }

    /// Returns the [`Touch`] input corresponding to the `id` if it has just been released.
    pub fn get_released(&self, id: u64) -> Option<&Touch> {
        self.just_released.get(&id)
    }

    /// Returns `true` if the input corresponding to the `id` has just been released.
    pub fn just_released(&self, id: u64) -> bool {
        self.just_released.contains_key(&id)
    }

    /// An iterator visiting every just released [`Touch`] input in arbitrary order.
    pub fn iter_just_released(&self) -> impl Iterator<Item = &Touch> {
        self.just_released.values()
    }

    /// Returns `true` if the input corresponding to the `id` has just been cancelled.
    pub fn just_cancelled(&self, id: u64) -> bool {
        self.just_cancelled.contains_key(&id)
    }

    /// An iterator visiting every just cancelled [`Touch`] input in arbitrary order.
    pub fn iter_just_cancelled(&self) -> impl Iterator<Item = &Touch> {
        self.just_cancelled.values()
    }

    /// Processes a [`TouchInput`] event by updating the `pressed`, `just_pressed`,
    /// `just_released`, and `just_cancelled` collections.
    fn process_touch_event(&mut self, event: &TouchInput) {
        match event.phase {
            TouchPhase::Started => {
                self.pressed.insert(event.id, event.into());
                self.just_pressed.insert(event.id, event.into());
            }
            TouchPhase::Moved => {
                if let Some(mut new_touch) = self.pressed.get(&event.id).cloned() {
                    new_touch.previous_position = new_touch.position;
                    new_touch.previous_force = new_touch.force;
                    new_touch.position = event.position;
                    new_touch.force = event.force;
                    self.pressed.insert(event.id, new_touch);
                }
            }
            TouchPhase::Ended => {
                self.just_released.insert(event.id, event.into());
                self.pressed.remove_entry(&event.id);
            }
            TouchPhase::Cancelled => {
                self.just_cancelled.insert(event.id, event.into());
                self.pressed.remove_entry(&event.id);
            }
        };
    }

    /// Clears the `just_pressed`, `just_released`, and `just_cancelled` collections.
    ///
    /// This is not clearing the `pressed` collection, because it could incorrectly mark
    /// a touch input as not pressed eventhough it is pressed. This could happen if the
    /// touch input is not moving for a single frame and would therefore be marked as
    /// not pressed, because this function is called on every single frame no matter
    /// if there was an event or not.
    fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.just_cancelled.clear();
    }
}
