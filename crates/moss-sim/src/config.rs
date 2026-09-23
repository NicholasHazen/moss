//! Validated dimensions for an authored run.

use std::{error::Error, fmt};

use bevy_ecs::prelude::Resource;

/// Dimensions in cells. Changing dimensions begins a new authored run.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    width: u32,
    height: u32,
}

impl WorldConfig {
    /// Keep the authored fixture legible and the bootstrap world deliberately small.
    pub fn new(width: u32, height: u32) -> Result<Self, WorldConfigError> {
        if !(8..=256).contains(&width) || !(8..=256).contains(&height) {
            return Err(WorldConfigError { width, height });
        }
        Ok(Self { width, height })
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            width: 32,
            height: 20,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfigError {
    width: u32,
    height: u32,
}

impl fmt::Display for WorldConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "world dimensions must each be 8..=256 cells; received {} × {}",
            self.width, self.height
        )
    }
}

impl Error for WorldConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rejects_dimensions_outside_the_supported_range() {
        for (width, height) in [(0, 20), (32, 0), (7, 8), (8, 7), (257, 20), (32, 257)] {
            assert!(WorldConfig::new(width, height).is_err());
        }
        assert!(WorldConfig::new(8, 8).is_ok());
        assert!(WorldConfig::new(256, 256).is_ok());
        assert_eq!(WorldConfig::default().width(), 32);
        assert_eq!(WorldConfig::default().height(), 20);
    }
}
