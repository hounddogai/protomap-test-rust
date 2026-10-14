use super::*;

use acme_kit::text;

/// Where a tiered failover adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFailoverAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredFailoverAdjustmentStatus {
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

impl std::fmt::Display for TieredFailoverAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered failover adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAdjustment {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub weight: f64,
    pub amount: f64,
    pub rotation_days: i64,
    pub capacity: i64,
    pub status: TieredFailoverAdjustmentStatus,
}

impl TieredFailoverAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFailoverAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFailoverAdjustmentStatus) -> bool {
        match self.status {
            TieredFailoverAdjustmentStatus::Planned => {
                match next {
                    TieredFailoverAdjustmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFailoverAdjustmentStatus::Scheduled => {
                return next == TieredFailoverAdjustmentStatus::Completed
                    || next == TieredFailoverAdjustmentStatus::Cancelled;
            }
            TieredFailoverAdjustmentStatus::InProgress => {
                return next == TieredFailoverAdjustmentStatus::Cancelled;
            }
            TieredFailoverAdjustmentStatus::Completed => {
                match next {
                    TieredFailoverAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredFailoverAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered failover adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFailoverAdjustment {
        TieredFailoverAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            weight: self.weight,
            amount: self.amount,
            rotation_days: self.rotation_days,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered failover adjustment in status planned.
pub fn new_tiered_failover_adjustment(code: &str, label: &str) -> TieredFailoverAdjustment {
    TieredFailoverAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        weight: 0.0,
        amount: 0.0,
        rotation_days: 0,
        capacity: 0,
        status: TieredFailoverAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_failover_adjustment("code-771", "label-563");
        assert_eq!(item.status, TieredFailoverAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_failover_adjustment("code-771", "label-563");
        if item.move_to(TieredFailoverAdjustmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFailoverAdjustmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_failover_adjustment("code-771", "label-563");
        item.status = TieredFailoverAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(TieredFailoverAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
