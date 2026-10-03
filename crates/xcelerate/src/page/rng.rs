//! Small non-cryptographic PRNG shared by the page's human-like motion code.

/// A linear congruential generator used to jitter mouse travel and click timing.
///
/// It is deliberately not cryptographic: it exists only to make synthetic input
/// look less mechanical.
pub(crate) struct Lcg {
    state: u64,
}

impl Lcg {
    pub(crate) fn new() -> Self {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let val = self.next();
        (val as f64) / (u64::MAX as f64)
    }

    pub(crate) fn range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_stays_within_bounds() {
        let mut rng = Lcg {
            state: 0x1234_5678_9abc_def0,
        };
        for _ in 0..10_000 {
            let value = rng.range(5.0, 10.0);
            assert!((5.0..=10.0).contains(&value), "out of range: {value}");
        }
    }

    #[test]
    fn same_seed_yields_same_sequence() {
        let mut a = Lcg { state: 42 };
        let mut b = Lcg { state: 42 };
        for _ in 0..64 {
            assert_eq!(a.next(), b.next());
        }
    }
}
