use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    time::Duration,
};

pub(crate) struct DosProtection {
    counters: HashMap<u16, PacketCounter>,
}

impl DosProtection {
    pub(crate) fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    const ONE_SECOND_IN_MS: i64 = Duration::from_secs(1).as_millis() as i64;

    pub(crate) fn evaluate_cmd(&mut self, cmd: u16, limit: u16, elapsed: i64) -> bool {
        if let Some(mut packet_counter) = self.counters.get_mut(&cmd) {
            let space = (packet_counter.last_receive_time + Self::ONE_SECOND_IN_MS) >= elapsed;

            if space {
                if packet_counter.amount >= limit {
                    return false;
                } else {
                    packet_counter.amount += 1;
                }
            } else {
                packet_counter.amount = 1;
            }

            packet_counter.last_receive_time = elapsed;

            true
        } else {
            self.counters.insert(cmd, PacketCounter::new(elapsed));
            true
        }
    }
}

impl Debug for DosProtection {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_map()
            .entries(self.counters.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

/// Policy if session exceed the limit.
#[derive(Debug, Clone)]
pub enum DosPolicy {
    Close,
    Log,
    None,
}

#[derive(Debug, Clone, Copy)]
struct PacketCounter {
    last_receive_time: i64,
    amount: u16,
}

impl PacketCounter {
    fn new(elapsed: i64) -> Self {
        Self {
            last_receive_time: elapsed,
            amount: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_evaluate_cmd() {
        let mut dos_protection = DosProtection::new();
        assert!(dos_protection.evaluate_cmd(10, 1, 0)); // 0 - 0(1000) True
        assert!(!dos_protection.evaluate_cmd(10, 1, 900)); // 1 - 900 (1000) False
        assert!(dos_protection.evaluate_cmd(10, 1, 1900)); // 1 - 1900 (1000) True
        assert!(!dos_protection.evaluate_cmd(10, 1, 1901)); // 1 - 1901 (2900) False
        assert!(dos_protection.evaluate_cmd(10, 1, 2901)); // 1 - 2901 (2901) True
        assert!(!dos_protection.evaluate_cmd(10, 1, 2905));
        assert!(dos_protection.evaluate_cmd(10, 1, 10000));
        assert!(!dos_protection.evaluate_cmd(10, 1, 10001));
    }
}
