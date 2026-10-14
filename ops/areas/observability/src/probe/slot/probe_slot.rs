use super::*;

use acme_kit::text;

/// Where a probe slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ProbeSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ProbeSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe slot as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSlot {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub version: i64,
    pub namespace: String,
    pub capacity: i64,
    pub weight: f64,
    pub ends_at: i64,
    pub principal: String,
    pub window_days: i64,
    pub ratio: f64,
    pub status: ProbeSlotStatus,
}

impl ProbeSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeSlotStatus) -> bool {
        match self.status {
            ProbeSlotStatus::Idle => {
                match next {
                    ProbeSlotStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeSlotStatus::Warming => {
                return next == ProbeSlotStatus::Ready;
            }
            ProbeSlotStatus::Ready => {
                match next {
                    ProbeSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeSlotStatus::Draining => {
                match next {
                    ProbeSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe slot", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeSlot {
        ProbeSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            version: self.version,
            namespace: self.namespace.clone(),
            capacity: self.capacity,
            weight: self.weight,
            ends_at: self.ends_at,
            principal: self.principal.clone(),
            window_days: self.window_days,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a probe slot in status idle.
pub fn new_probe_slot(code: &str, label: &str) -> ProbeSlot {
    ProbeSlot {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        version: 0,
        namespace: "".to_string(),
        capacity: 0,
        weight: 0.0,
        ends_at: 0,
        principal: "".to_string(),
        window_days: 0,
        ratio: 0.0,
        status: ProbeSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_probe_slot("code-881", "label-730");
        assert_eq!(item.status, ProbeSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_probe_slot("code-881", "label-730");
        if item.move_to(ProbeSlotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeSlotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_probe_slot("code-881", "label-730");
        item.status = ProbeSlotStatus::Stopped;
        assert!(item.move_to(ProbeSlotStatus::Idle).is_err(), "expected move to fail");
    }
}
