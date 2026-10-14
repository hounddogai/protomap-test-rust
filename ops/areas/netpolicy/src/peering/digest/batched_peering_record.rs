use super::*;

use acme_kit::text;

/// Where a batched peering record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPeeringRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPeeringRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPeeringRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched peering record as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecord {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub burst: i64,
    pub budget: f64,
    pub rate: f64,
    pub cipher: String,
    pub namespace: String,
    pub status: BatchedPeeringRecordStatus,
}

impl BatchedPeeringRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPeeringRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPeeringRecordStatus) -> bool {
        match self.status {
            BatchedPeeringRecordStatus::Proposed => {
                match next {
                    BatchedPeeringRecordStatus::Rejected | BatchedPeeringRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPeeringRecordStatus::Approved => {
                match next {
                    BatchedPeeringRecordStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPeeringRecordStatus::Rejected => {
                match next {
                    BatchedPeeringRecordStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedPeeringRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched peering record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched peering record", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPeeringRecord {
        BatchedPeeringRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            burst: self.burst,
            budget: self.budget,
            rate: self.rate,
            cipher: self.cipher.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched peering record in status proposed.
pub fn new_batched_peering_record(code: &str, title: &str) -> BatchedPeeringRecord {
    BatchedPeeringRecord {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        burst: 0,
        budget: 0.0,
        rate: 0.0,
        cipher: "".to_string(),
        namespace: "".to_string(),
        status: BatchedPeeringRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_peering_record("code-328", "title-754");
        assert_eq!(item.status, BatchedPeeringRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_peering_record("code-328", "title-754");
        if item.move_to(BatchedPeeringRecordStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPeeringRecordStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_peering_record("code-328", "title-754");
        item.status = BatchedPeeringRecordStatus::Withdrawn;
        assert!(
            item.move_to(BatchedPeeringRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
