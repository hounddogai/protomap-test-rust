use super::*;

use acme_kit::text;

/// Where a rolling egress entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEgressEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingEgressEntryStatus {
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

impl std::fmt::Display for RollingEgressEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling egress entry as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntry {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub source: String,
    pub floor_value: f64,
    pub status: RollingEgressEntryStatus,
}

impl RollingEgressEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEgressEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEgressEntryStatus) -> bool {
        match self.status {
            RollingEgressEntryStatus::Idle => {
                return next == RollingEgressEntryStatus::Stopped;
            }
            RollingEgressEntryStatus::Warming => {
                match next {
                    RollingEgressEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressEntryStatus::Ready => {
                match next {
                    RollingEgressEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEgressEntryStatus::Draining => {
                return next == RollingEgressEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEgressEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling egress entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling egress entry", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rate + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEgressEntry {
        RollingEgressEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            source: self.source.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling egress entry in status idle.
pub fn new_rolling_egress_entry(code: &str, label: &str) -> RollingEgressEntry {
    RollingEgressEntry {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        source: "".to_string(),
        floor_value: 0.0,
        status: RollingEgressEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_egress_entry("code-129", "label-972");
        assert_eq!(item.status, RollingEgressEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_rolling_egress_entry("code-129", "label-972");
        if item.move_to(RollingEgressEntryStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingEgressEntryStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_egress_entry("code-129", "label-972");
        item.status = RollingEgressEntryStatus::Stopped;
        assert!(
            item.move_to(RollingEgressEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
