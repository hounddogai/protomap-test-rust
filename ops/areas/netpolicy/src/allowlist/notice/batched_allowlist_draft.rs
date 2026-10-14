use super::*;

use acme_kit::text;

/// Where a batched allowlist draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedAllowlistDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched allowlist draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistDraft {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub quota_units: i64,
    pub channel: String,
    pub window_days: i64,
    pub status: BatchedAllowlistDraftStatus,
}

impl BatchedAllowlistDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistDraftStatus) -> bool {
        match self.status {
            BatchedAllowlistDraftStatus::Draft => {
                match next {
                    BatchedAllowlistDraftStatus::Pending | BatchedAllowlistDraftStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistDraftStatus::Pending => {
                match next {
                    BatchedAllowlistDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistDraftStatus::Active => {
                return next == BatchedAllowlistDraftStatus::Suspended;
            }
            BatchedAllowlistDraftStatus::Suspended => {
                return next == BatchedAllowlistDraftStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAllowlistDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.weight - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistDraft {
        BatchedAllowlistDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched allowlist draft in status draft.
pub fn new_batched_allowlist_draft(code: &str, name: &str) -> BatchedAllowlistDraft {
    BatchedAllowlistDraft {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        quota_units: 0,
        channel: "".to_string(),
        window_days: 0,
        status: BatchedAllowlistDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_allowlist_draft("code-423", "name-605");
        assert_eq!(item.status, BatchedAllowlistDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_allowlist_draft("code-423", "name-605");
        if item.move_to(BatchedAllowlistDraftStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistDraftStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_allowlist_draft("code-423", "name-605");
        item.status = BatchedAllowlistDraftStatus::Closed;
        assert!(
            item.move_to(BatchedAllowlistDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
