use super::*;

use acme_kit::text;

/// Where a batched token entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTokenEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedTokenEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedTokenEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched token entry as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenEntry {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub quota_units: i64,
    pub expires_at: i64,
    pub status: BatchedTokenEntryStatus,
}

impl BatchedTokenEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTokenEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTokenEntryStatus) -> bool {
        match self.status {
            BatchedTokenEntryStatus::Proposed => {
                match next {
                    BatchedTokenEntryStatus::Approved | BatchedTokenEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTokenEntryStatus::Approved => {
                match next {
                    BatchedTokenEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTokenEntryStatus::Rejected => {
                match next {
                    BatchedTokenEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedTokenEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched token entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched token entry", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTokenEntry {
        BatchedTokenEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            quota_units: self.quota_units,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched token entry in status proposed.
pub fn new_batched_token_entry(code: &str, title: &str) -> BatchedTokenEntry {
    BatchedTokenEntry {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        quota_units: 0,
        expires_at: 0,
        status: BatchedTokenEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_token_entry("code-861", "title-787");
        assert_eq!(item.status, BatchedTokenEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_token_entry("code-861", "title-787");
        if item.move_to(BatchedTokenEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTokenEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_token_entry("code-861", "title-787");
        item.status = BatchedTokenEntryStatus::Withdrawn;
        assert!(
            item.move_to(BatchedTokenEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
