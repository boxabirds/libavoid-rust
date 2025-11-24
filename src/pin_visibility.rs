//! Pin visibility computation (Task #14)
//!
//! Handles visibility between connection pins on shapes.
//! C++ Reference: libavoid/shape.cpp:330-420

use crate::shape::ConnectionPin;
use std::collections::HashMap;

/// Pin visibility handler
pub struct PinVisibility {
    /// Cache of pin-to-pin visibility
    visibility_cache: HashMap<(u32, u32), bool>,
}

impl PinVisibility {
    /// Create new pin visibility handler
    pub fn new() -> Self {
        PinVisibility {
            visibility_cache: HashMap::new(),
        }
    }

    /// Check if two pins have visibility to each other
    pub fn pins_have_visibility(&self, pin1_id: u32, pin2_id: u32) -> bool {
        // Check cache
        let key = if pin1_id < pin2_id {
            (pin1_id, pin2_id)
        } else {
            (pin2_id, pin1_id)
        };

        self.visibility_cache.get(&key).copied().unwrap_or(true)
    }

    /// Update visibility between two pins
    pub fn set_pin_visibility(&mut self, pin1_id: u32, pin2_id: u32, visible: bool) {
        let key = if pin1_id < pin2_id {
            (pin1_id, pin2_id)
        } else {
            (pin2_id, pin1_id)
        };

        self.visibility_cache.insert(key, visible);
    }

    /// Filter pins by class compatibility
    pub fn filter_by_class(pins: &[ConnectionPin], target_class: u32) -> Vec<&ConnectionPin> {
        pins.iter()
            .filter(|pin| pin.class_id == target_class || pin.class_id == 0 || target_class == 0)
            .collect()
    }

    /// Check if pin direction is compatible with target direction
    pub fn direction_compatible(pin: &ConnectionPin, direction: u32) -> bool {
        if pin.directions == 0 {
            return true; // All directions allowed
        }
        (pin.directions & direction) != 0
    }

    /// Clear visibility cache
    pub fn clear_cache(&mut self) {
        self.visibility_cache.clear();
    }

    /// Get number of cached visibility entries
    pub fn cache_size(&self) -> usize {
        self.visibility_cache.len()
    }
}
