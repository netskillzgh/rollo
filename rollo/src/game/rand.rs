use lazy_static::lazy_static;
use parking_lot::Mutex;
use rand::{Rng, SeedableRng};
#[cfg(target_pointer_width = "32")]
use rand_pcg::Lcg64Xsh32;
#[cfg(target_pointer_width = "64")]
use rand_pcg::Mcg128Xsl64;

#[cfg(target_pointer_width = "64")]
lazy_static! {
    static ref RNG: Mutex<Mcg128Xsl64> = Mutex::new(Mcg128Xsl64::from_entropy());
}

#[cfg(target_pointer_width = "32")]
lazy_static! {
    static ref RNG: Mutex<Lcg64Xsh32> = Mutex::new(Lcg64Xsh32::from_entropy());
}

/// Roll with a chance.
pub fn roll(chance: f32) -> (bool, f32) {
    let r = rand_chance();
    (chance > r, r)
}

fn rand_chance() -> f32 {
    return RNG.lock().gen_range(0.0..=100.0);
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
        assert!(roll(100f32).0);
    }
}
