
use std::{hash::Hash, collections::HashSet};

use bevy_ecs::system::Resource;
#[derive(Debug, Clone, Resource)]
pub struct Input<T: Copy + Eq + Hash + Send + Sync + 'static> {
    /// A collection of every button that is currently being pressed.
    pressed: HashSet<T>,
    /// A collection of every button that has just been pressed.
    just_pressed: HashSet<T>,
    /// A collection of every button that has just been released.
    just_released: HashSet<T>,
}

impl<T: Copy + Eq + Hash + Send + Sync + 'static> Default for Input<T> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }
}

impl<T> Input<T>
where
    T: Copy + Eq + Hash + Send + Sync + 'static,
{
    /// Registers a press for the given `input`.
    pub fn press(&mut self, input: T) {
        // Returns `true` if the `input` wasn't pressed.
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    /// Returns `true` if the `input` has been pressed.
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has been pressed.
    pub fn any_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.pressed(it))
    }

    /// Registers a release for the given `input`.
    pub fn release(&mut self, input: T) {
        // Returns `true` if the `input` was pressed.
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    /// Registers a release for all currently pressed inputs.
    pub fn release_all(&mut self) {
        // Move all items from pressed into just_released
        self.just_released.extend(self.pressed.drain());
    }

    /// Returns `true` if the `input` has just been pressed.
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has just been pressed.
    pub fn any_just_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.just_pressed(it))
    }

    /// Clears the `just_pressed` state of the `input` and returns `true` if the `input` has just been pressed.
    ///
    /// Future calls to [`Input::just_pressed`] for the given input will return false until a new press event occurs.
    pub fn clear_just_pressed(&mut self, input: T) -> bool {
        self.just_pressed.remove(&input)
    }

    /// Returns `true` if the `input` has just been released.
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    /// Returns `true` if any item in `inputs` has just been released.
    pub fn any_just_released(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|it| self.just_released(it))
    }

    /// Clears the `just_released` state of the `input` and returns `true` if the `input` has just been released.
    ///
    /// Future calls to [`Input::just_released`] for the given input will return false until a new release event occurs.
    pub fn clear_just_released(&mut self, input: T) -> bool {
        self.just_released.remove(&input)
    }

    /// Clears the `pressed`, `just_pressed` and `just_released` data of the `input`.
    pub fn reset(&mut self, input: T) {
        self.pressed.remove(&input);
        self.just_pressed.remove(&input);
        self.just_released.remove(&input);
    }

    /// Clears the `pressed`, `just_pressed`, and `just_released` data for every input.
    ///
    /// See also [`Input::clear`] for simulating elapsed time steps.
    pub fn reset_all(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Clears the `just pressed` and `just released` data for every input.
    ///
    /// See also [`Input::reset_all`] for a full reset.
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// An iterator visiting every pressed input in arbitrary order.
    pub fn get_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.pressed.iter()
    }

    /// An iterator visiting every just pressed input in arbitrary order.
    pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_pressed.iter()
    }

    /// An iterator visiting every just released input in arbitrary order.
    pub fn get_just_released(&self) -> impl ExactSizeIterator<Item = &T> {
        self.just_released.iter()
    }
}
