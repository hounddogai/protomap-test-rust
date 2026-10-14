use super::*;

use acme_kit::text;

/// Where a rolling federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFederationCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingFederationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RollingFederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling federation cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationCycle {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub floor_value: f64,
    pub summary: String,
    pub pinned: bool,
    pub status: RollingFederationCycleStatus,
}

impl RollingFederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFederationCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFederationCycleStatus) -> bool {
        match self.status {
            RollingFederationCycleStatus::Planned => {
                match next {
                    RollingFederationCycleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFederationCycleStatus::Scheduled => {
                return next == RollingFederationCycleStatus::InProgress
                    || next == RollingFederationCycleStatus::Completed;
            }
            RollingFederationCycleStatus::InProgress => {
                return next == RollingFederationCycleStatus::Completed
                    || next == RollingFederationCycleStatus::Cancelled;
            }
            RollingFederationCycleStatus::Completed => {
                return next == RollingFederationCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingFederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling federation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFederationCycle {
        RollingFederationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            floor_value: self.floor_value,
            summary: self.summary.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling federation cycle in status planned.
pub fn new_rolling_federation_cycle(code: &str, label: &str) -> RollingFederationCycle {
    RollingFederationCycle {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        floor_value: 0.0,
        summary: "".to_string(),
        pinned: false,
        status: RollingFederationCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_federation_cycle("code-9", "label-619");
        assert_eq!(item.status, RollingFederationCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rolling_federation_cycle("code-9", "label-619");
        if item.move_to(RollingFederationCycleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFederationCycleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_federation_cycle("code-9", "label-619");
        item.status = RollingFederationCycleStatus::Cancelled;
        assert!(
            item.move_to(RollingFederationCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
