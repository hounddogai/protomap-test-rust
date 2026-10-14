use super::*;

use acme_kit::text;

/// Where a weighted failover slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFailoverSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedFailoverSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedFailoverSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted failover slot as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverSlot {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub notes: String,
    pub host: String,
    pub weight: f64,
    pub status: WeightedFailoverSlotStatus,
}

impl WeightedFailoverSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFailoverSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFailoverSlotStatus) -> bool {
        match self.status {
            WeightedFailoverSlotStatus::Proposed => {
                return next == WeightedFailoverSlotStatus::Rejected || next == WeightedFailoverSlotStatus::Withdrawn;
            }
            WeightedFailoverSlotStatus::Approved => {
                match next {
                    WeightedFailoverSlotStatus::Rejected | WeightedFailoverSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverSlotStatus::Rejected => {
                return next == WeightedFailoverSlotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFailoverSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted failover slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted failover slot", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.revision as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFailoverSlot {
        WeightedFailoverSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            notes: self.notes.clone(),
            host: self.host.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted failover slot in status proposed.
pub fn new_weighted_failover_slot(code: &str, label: &str) -> WeightedFailoverSlot {
    WeightedFailoverSlot {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        notes: "".to_string(),
        host: "".to_string(),
        weight: 0.0,
        status: WeightedFailoverSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_failover_slot("code-906", "label-931");
        assert_eq!(item.status, WeightedFailoverSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_weighted_failover_slot("code-906", "label-931");
        if item.move_to(WeightedFailoverSlotStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFailoverSlotStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_failover_slot("code-906", "label-931");
        item.status = WeightedFailoverSlotStatus::Withdrawn;
        assert!(
            item.move_to(WeightedFailoverSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
