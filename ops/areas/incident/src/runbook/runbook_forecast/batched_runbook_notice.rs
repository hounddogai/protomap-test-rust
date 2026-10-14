use super::*;

use acme_kit::text;

/// Where a batched runbook notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedRunbookNoticeStatus {
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

impl std::fmt::Display for BatchedRunbookNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched runbook notice as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNotice {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub source: String,
    pub quota_units: i64,
    pub fingerprint: String,
    pub status: BatchedRunbookNoticeStatus,
}

impl BatchedRunbookNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookNoticeStatus) -> bool {
        match self.status {
            BatchedRunbookNoticeStatus::Draft => {
                return next == BatchedRunbookNoticeStatus::Pending || next == BatchedRunbookNoticeStatus::Active;
            }
            BatchedRunbookNoticeStatus::Pending => {
                return next == BatchedRunbookNoticeStatus::Active;
            }
            BatchedRunbookNoticeStatus::Active => {
                match next {
                    BatchedRunbookNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookNoticeStatus::Suspended => {
                match next {
                    BatchedRunbookNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedRunbookNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> BatchedRunbookNotice {
        BatchedRunbookNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            quota_units: self.quota_units,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a batched runbook notice in status draft.
pub fn new_batched_runbook_notice(code: &str, label: &str) -> BatchedRunbookNotice {
    BatchedRunbookNotice {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        source: "".to_string(),
        quota_units: 0,
        fingerprint: "".to_string(),
        status: BatchedRunbookNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_runbook_notice("code-5", "label-571");
        assert_eq!(item.status, BatchedRunbookNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_runbook_notice("code-5", "label-571");
        if item.move_to(BatchedRunbookNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRunbookNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_runbook_notice("code-5", "label-571");
        item.status = BatchedRunbookNoticeStatus::Closed;
        assert!(
            item.move_to(BatchedRunbookNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
