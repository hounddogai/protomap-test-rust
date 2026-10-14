use super::*;

use acme_kit::text;

/// Where a canary batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CanaryBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CanaryBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary batch as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatch {
    pub code: String,
    pub label: String,
    pub region: String,
    pub source: String,
    pub priority: i64,
    pub amount: f64,
    pub status: CanaryBatchStatus,
}

impl CanaryBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryBatchStatus) -> bool {
        match self.status {
            CanaryBatchStatus::Requested => {
                return next == CanaryBatchStatus::Granted;
            }
            CanaryBatchStatus::Granted => {
                return next == CanaryBatchStatus::Revoked;
            }
            CanaryBatchStatus::Revoked => {
                match next {
                    CanaryBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: CanaryBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "canary batch", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryBatch {
        CanaryBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            source: self.source.clone(),
            priority: self.priority,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a canary batch in status requested.
pub fn new_canary_batch(code: &str, label: &str) -> CanaryBatch {
    CanaryBatch {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        source: "".to_string(),
        priority: 0,
        amount: 0.0,
        status: CanaryBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_canary_batch("code-62", "label-57");
        assert_eq!(item.status, CanaryBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_canary_batch("code-62", "label-57");
        if item.move_to(CanaryBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_canary_batch("code-62", "label-57");
        item.status = CanaryBatchStatus::Expired;
        assert!(
            item.move_to(CanaryBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
