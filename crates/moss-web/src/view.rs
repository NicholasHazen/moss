//! Camera mathematics in CSS pixels. These values never enter simulation rules.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

pub struct MapView {
    pub center: [f32; 2],
    pub units_per_pixel: f32,
    pub size: [f32; 2],
    pub fitted: bool,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            center: [16.0, 10.0],
            units_per_pixel: 0.05,
            size: [1.0, 1.0],
            fitted: true,
        }
    }
}

impl MapView {
    pub fn fit(&mut self, world_width: u32, world_height: u32) {
        self.center = [world_width as f32 / 2.0, world_height as f32 / 2.0];
        self.units_per_pixel = ((world_width as f32 + 4.0) / self.size[0])
            .max((world_height as f32 + 4.0) / self.size[1])
            .clamp(0.005, 2.0);
        self.fitted = true;
    }

    pub fn world_at(&self, x: f32, y: f32) -> [f32; 2] {
        [
            self.center[0] + (x - self.size[0] / 2.0) * self.units_per_pixel,
            self.center[1] - (y - self.size[1] / 2.0) * self.units_per_pixel,
        ]
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.center[0] -= dx * self.units_per_pixel;
        self.center[1] += dy * self.units_per_pixel;
        self.fitted = false;
    }

    pub fn zoom(&mut self, x: f32, y: f32, factor: f32) {
        let before = self.world_at(x, y);
        self.units_per_pixel = (self.units_per_pixel * factor).clamp(0.005, 2.0);
        let after = self.world_at(x, y);
        self.center[0] += before[0] - after[0];
        self.center[1] += before[1] - after[1];
        self.fitted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_pointer_anchored_even_at_limits() {
        let mut view = MapView {
            size: [800.0, 600.0],
            ..Default::default()
        };
        view.fit(32, 20);
        let before = view.world_at(123.0, 456.0);
        for factor in [0.5, 0.0001, 100_000.0] {
            view.zoom(123.0, 456.0, factor);
            let after = view.world_at(123.0, 456.0);
            assert!((before[0] - after[0]).abs() < 0.0001);
            assert!((before[1] - after[1]).abs() < 0.0001);
        }
    }

    #[test]
    fn pan_tracks_screen_coordinates_and_fit_recovers_world() {
        let mut view = MapView {
            size: [800.0, 600.0],
            ..Default::default()
        };
        view.fit(32, 20);
        let point = view.world_at(100.0, 100.0);
        view.pan(40.0, -20.0);
        assert_eq!(point, view.world_at(140.0, 80.0));
        view.fit(32, 20);
        assert_eq!(view.center, [16.0, 10.0]);
    }
}
