use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    time::Duration,
};

pub(crate) struct DosProtection {
    counters: HashMap<u16, PacketCounter>,
    global_counter: GlobalCounter,
}

const ONE_SECOND_IN_MS: i64 = Duration::from_secs(1).as_millis() as i64;

impl DosProtection {
    pub(crate) fn new() -> Self {
        Self {
            counters: HashMap::new(),
            global_counter: GlobalCounter::new(),
        }
    }

    pub(crate) fn evaluate_global_limit(
        &mut self,
        time: i64,
        size: u32,
        limit_size: u32,
        limit_amount: u16,
    ) -> bool {
        let space = (self.global_counter.last_receive_time + ONE_SECOND_IN_MS) >= time;

        if self.global_counter.last_receive_time == 0 {
            self.global_counter.last_receive_time = time;
            self.global_counter.size = size;
            self.global_counter.amount = 1;

            return true;
        }

        if space {
            if self.global_counter.size >= limit_size || self.global_counter.amount >= limit_amount
            {
                return false;
            }

            self.global_counter.size += size;
            self.global_counter.amount += 1;
        } else {
            self.global_counter.amount = 1;
            self.global_counter.size = size;
            self.global_counter.last_receive_time = time;
        }

        true
    }

    pub(crate) fn evaluate_cmd(&mut self, cmd: u16, limit: u16, time: i64) -> bool {
        if let Some(mut packet_counter) = self.counters.get_mut(&cmd) {
            let space = (packet_counter.last_receive_time + ONE_SECOND_IN_MS) >= time;

            if space {
                if packet_counter.amount >= limit {
                    return false;
                } else {
                    packet_counter.amount += 1;
                }
            } else {
                packet_counter.amount = 1;
                packet_counter.last_receive_time = time;
            }

            true
        } else {
            self.counters.insert(cmd, PacketCounter::new(time));
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

struct GlobalCounter {
    last_receive_time: i64,
    amount: u16,
    size: u32,
}

impl GlobalCounter {
    fn new() -> Self {
        Self {
            amount: 0,
            size: 0,
            last_receive_time: 0,
        }
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
    fn new(time: i64) -> Self {
        Self {
            last_receive_time: time,
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
        assert!(dos_protection.evaluate_cmd(10, 1, 0));
        assert!(!dos_protection.evaluate_cmd(10, 1, 900));
        assert!(dos_protection.evaluate_cmd(10, 1, 1900));
        assert!(!dos_protection.evaluate_cmd(10, 1, 1901));
        assert!(dos_protection.evaluate_cmd(10, 1, 2901));
        assert!(!dos_protection.evaluate_cmd(10, 1, 2905));
        assert!(dos_protection.evaluate_cmd(10, 1, 10000));
        assert!(!dos_protection.evaluate_cmd(10, 1, 10001));

        let mut dos_protection = DosProtection::new();
        assert!(dos_protection.evaluate_cmd(10, 1, 0));
        assert!(!dos_protection.evaluate_cmd(10, 1, 1000));
    }

    #[test]
    fn test_evaluate_global() {
        // Size
        let mut dos_protection = DosProtection::new();
        assert!(dos_protection.evaluate_global_limit(1000, 100, 200, 4));
        assert!(dos_protection.evaluate_global_limit(1500, 100, 200, 4));
        assert!(!dos_protection.evaluate_global_limit(1999, 100, 200, 4));

        // Amount
        let mut dos_protection = DosProtection::new();
        assert!(dos_protection.evaluate_global_limit(1000, 100, 500, 3));
        assert!(dos_protection.evaluate_global_limit(1500, 100, 500, 3));
        assert!(dos_protection.evaluate_global_limit(1998, 100, 500, 3));
        assert!(!dos_protection.evaluate_global_limit(1998, 100, 500, 3));

        // Both
        assert!(dos_protection.evaluate_global_limit(2010, 100, 200, 2));
        assert!(dos_protection.evaluate_global_limit(2500, 100, 200, 2));
        assert!(!dos_protection.evaluate_global_limit(2600, 100, 200, 2));
    }
}
