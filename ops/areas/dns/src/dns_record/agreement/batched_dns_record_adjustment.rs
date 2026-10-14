use super::*;

use acme_kit::text;

/// Where a batched dns record adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDnsRecordAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedDnsRecordAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BatchedDnsRecordAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched dns record adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordAdjustment {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub floor_value: f64,
    pub description: String,
    pub pinned: bool,
    pub amount: f64,
    pub status: BatchedDnsRecordAdjustmentStatus,
}

impl BatchedDnsRecordAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDnsRecordAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDnsRecordAdjustmentStatus) -> bool {
        match self.status {
            BatchedDnsRecordAdjustmentStatus::Open => {
                return next == BatchedDnsRecordAdjustmentStatus::Resolved;
            }
            BatchedDnsRecordAdjustmentStatus::InReview => {
                return next == BatchedDnsRecordAdjustmentStatus::Resolved
                    || next == BatchedDnsRecordAdjustmentStatus::Archived;
            }
            BatchedDnsRecordAdjustmentStatus::Resolved => {
                match next {
                    BatchedDnsRecordAdjustmentStatus::Reopened | BatchedDnsRecordAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDnsRecordAdjustmentStatus::Reopened => {
                match next {
                    BatchedDnsRecordAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: BatchedDnsRecordAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched dns record adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched dns record adjustment", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.amount + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDnsRecordAdjustment {
        BatchedDnsRecordAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            floor_value: self.floor_value,
            description: self.description.clone(),
            pinned: self.pinned,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched dns record adjustment in status open.
pub fn new_batched_dns_record_adjustment(code: &str, name: &str) -> BatchedDnsRecordAdjustment {
    BatchedDnsRecordAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        floor_value: 0.0,
        description: "".to_string(),
        pinned: false,
        amount: 0.0,
        status: BatchedDnsRecordAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_dns_record_adjustment("code-982", "name-212");
        assert_eq!(item.status, BatchedDnsRecordAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_batched_dns_record_adjustment("code-982", "name-212");
        if item.move_to(BatchedDnsRecordAdjustmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDnsRecordAdjustmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_dns_record_adjustment("code-982", "name-212");
        item.status = BatchedDnsRecordAdjustmentStatus::Archived;
        assert!(
            item.move_to(BatchedDnsRecordAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
