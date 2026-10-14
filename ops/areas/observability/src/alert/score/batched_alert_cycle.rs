use super::*;

use acme_kit::text;

/// Where a batched alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAlertCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedAlertCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedAlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched alert cycle as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAlertCycle {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub channel: String,
    pub tenant_id: String,
    pub unit: String,
    pub starts_at: i64,
    pub notes: String,
    pub status: BatchedAlertCycleStatus,
}

impl BatchedAlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAlertCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAlertCycleStatus) -> bool {
        match self.status {
            BatchedAlertCycleStatus::Proposed => {
                match next {
                    BatchedAlertCycleStatus::Approved | BatchedAlertCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAlertCycleStatus::Approved => {
                return next == BatchedAlertCycleStatus::Rejected || next == BatchedAlertCycleStatus::Withdrawn;
            }
            BatchedAlertCycleStatus::Rejected => {
                match next {
                    BatchedAlertCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedAlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> BatchedAlertCycle {
        BatchedAlertCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a batched alert cycle in status proposed.
pub fn new_batched_alert_cycle(code: &str, label: &str) -> BatchedAlertCycle {
    BatchedAlertCycle {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        channel: "".to_string(),
        tenant_id: "".to_string(),
        unit: "".to_string(),
        starts_at: 0,
        notes: "".to_string(),
        status: BatchedAlertCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_alert_cycle("code-905", "label-944");
        assert_eq!(item.status, BatchedAlertCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_alert_cycle("code-905", "label-944");
        if item.move_to(BatchedAlertCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAlertCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_alert_cycle("code-905", "label-944");
        item.status = BatchedAlertCycleStatus::Withdrawn;
        assert!(
            item.move_to(BatchedAlertCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
