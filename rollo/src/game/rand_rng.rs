use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use std::sync::atomic::{AtomicU64, Ordering};

static SEED: AtomicU64 = AtomicU64::new(0);

/// ## Roll with a chance
/// ```rust, no_run
/// use rollo::game::roll;
///
/// // 100%
/// let (ok, result) = roll(100.0);
/// assert!(ok);
/// // 0%
/// let (ok, result) = roll(0.0);
/// assert!(!ok);
/// ```
pub fn roll(chance: f32) -> (bool, f32) {
    let r = rand_chance();
    (chance >= r, r)
}

fn rand_chance() -> f32 {
    let seed = SEED.fetch_add(1, Ordering::Relaxed);

    Xoshiro256Plus::seed_from_u64(seed).gen_range(0.0..=100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_chance() {
        let result = rand_chance();
        assert!((0f32..=100f32).contains(&result));
    }

    #[test]
    fn test_roll() {
        assert!(!roll(0f32).0);
        // Might be false...
        assert!(roll(99f32).0);
        assert!(roll(0f32).1 >= 0.0);
        assert!(roll(100f32).0);
        assert!(roll(100f32).1 <= 100.0);
    }
}
