use super::*;

use acme_kit::text;

/// Where a batched limit mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedLimitMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedLimitMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedLimitMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched limit mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMapping {
    pub code: String,
    pub label: String,
    pub country: String,
    pub batch_size: i64,
    pub window_days: i64,
    pub status: BatchedLimitMappingStatus,
}

impl BatchedLimitMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedLimitMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedLimitMappingStatus) -> bool {
        match self.status {
            BatchedLimitMappingStatus::Requested => {
                return next == BatchedLimitMappingStatus::Granted || next == BatchedLimitMappingStatus::Expired;
            }
            BatchedLimitMappingStatus::Granted => {
                match next {
                    BatchedLimitMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitMappingStatus::Revoked => {
                return next == BatchedLimitMappingStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedLimitMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched limit mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched limit mapping", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.window_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedLimitMapping {
        BatchedLimitMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            batch_size: self.batch_size,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched limit mapping in status requested.
pub fn new_batched_limit_mapping(code: &str, label: &str) -> BatchedLimitMapping {
    BatchedLimitMapping {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        batch_size: 0,
        window_days: 0,
        status: BatchedLimitMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_limit_mapping("code-53", "label-824");
        assert_eq!(item.status, BatchedLimitMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_limit_mapping("code-53", "label-824");
        if item.move_to(BatchedLimitMappingStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedLimitMappingStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_limit_mapping("code-53", "label-824");
        item.status = BatchedLimitMappingStatus::Expired;
        assert!(
            item.move_to(BatchedLimitMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
