use super::*;

use acme_kit::text;

/// Where a failover adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FailoverAdjustmentStatus {
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

impl std::fmt::Display for FailoverAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover adjustment as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAdjustment {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub visible: bool,
    pub sequence: i64,
    pub external_ref: String,
    pub window_days: i64,
    pub status: FailoverAdjustmentStatus,
}

impl FailoverAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverAdjustmentStatus) -> bool {
        match self.status {
            FailoverAdjustmentStatus::Planned => {
                match next {
                    FailoverAdjustmentStatus::InProgress | FailoverAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverAdjustmentStatus::Scheduled => {
                return next == FailoverAdjustmentStatus::Cancelled;
            }
            FailoverAdjustmentStatus::InProgress => {
                return next == FailoverAdjustmentStatus::Completed;
            }
            FailoverAdjustmentStatus::Completed => {
                return next == FailoverAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover adjustment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverAdjustment {
        FailoverAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            visible: self.visible,
            sequence: self.sequence,
            external_ref: self.external_ref.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover adjustment in status planned.
pub fn new_failover_adjustment(code: &str, label: &str) -> FailoverAdjustment {
    FailoverAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        visible: false,
        sequence: 0,
        external_ref: "".to_string(),
        window_days: 0,
        status: FailoverAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_failover_adjustment("code-266", "label-522");
        assert_eq!(item.status, FailoverAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_failover_adjustment("code-266", "label-522");
        if item.move_to(FailoverAdjustmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverAdjustmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_failover_adjustment("code-266", "label-522");
        item.status = FailoverAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(FailoverAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
