use super::*;

use acme_kit::text;

/// Where a capped peering record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPeeringRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedPeeringRecordStatus {
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

impl std::fmt::Display for CappedPeeringRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped peering record as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringRecord {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub notes: String,
    pub port: i64,
    pub grace_days: i64,
    pub revision: i64,
    pub floor_value: f64,
    pub owner_id: String,
    pub status: CappedPeeringRecordStatus,
}

impl CappedPeeringRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPeeringRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPeeringRecordStatus) -> bool {
        match self.status {
            CappedPeeringRecordStatus::Planned => {
                match next {
                    CappedPeeringRecordStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPeeringRecordStatus::Scheduled => {
                match next {
                    CappedPeeringRecordStatus::InProgress | CappedPeeringRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPeeringRecordStatus::InProgress => {
                return next == CappedPeeringRecordStatus::Completed || next == CappedPeeringRecordStatus::Cancelled;
            }
            CappedPeeringRecordStatus::Completed => {
                return next == CappedPeeringRecordStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPeeringRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped peering record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped peering record", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ceiling * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPeeringRecord {
        CappedPeeringRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            notes: self.notes.clone(),
            port: self.port,
            grace_days: self.grace_days,
            revision: self.revision,
            floor_value: self.floor_value,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped peering record in status planned.
pub fn new_capped_peering_record(code: &str, label: &str) -> CappedPeeringRecord {
    CappedPeeringRecord {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        notes: "".to_string(),
        port: 0,
        grace_days: 0,
        revision: 0,
        floor_value: 0.0,
        owner_id: "".to_string(),
        status: CappedPeeringRecordStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_peering_record("code-554", "label-372");
        assert_eq!(item.status, CappedPeeringRecordStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_capped_peering_record("code-554", "label-372");
        if item.move_to(CappedPeeringRecordStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPeeringRecordStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_peering_record("code-554", "label-372");
        item.status = CappedPeeringRecordStatus::Cancelled;
        assert!(
            item.move_to(CappedPeeringRecordStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
