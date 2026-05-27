#[derive(Clone, Debug)]
pub struct SimulationRng {
    state: u64,
}

impl Default for SimulationRng {
    fn default() -> Self {
        Self::with_seed(0x5446_4c5f_4352_4153)
    }
}

impl SimulationRng {
    pub fn with_seed(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        assert!(min < max, "invalid random range");
        min + (self.next_u32() % (max - min) as u32) as i32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        assert!(min < max, "invalid random range");
        let unit = self.next_u32() as f32 / u32::MAX as f32;
        min + (max - min) * unit
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_rng_is_seeded_and_bounded() {
        let mut first = SimulationRng::with_seed(42);
        let mut second = SimulationRng::with_seed(42);

        assert_eq!(first.range_i32(-1, 2), second.range_i32(-1, 2));
        let value = first.range_f32(0.0, 100.0);
        assert!((0.0..100.0).contains(&value));
    }
}
