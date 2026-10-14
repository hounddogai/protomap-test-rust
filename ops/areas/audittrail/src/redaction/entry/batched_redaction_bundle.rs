use super::*;

use acme_kit::text;

/// Where a batched redaction bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRedactionBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRedactionBundleStatus {
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

impl std::fmt::Display for BatchedRedactionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched redaction bundle as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundle {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub port: i64,
    pub created_at: i64,
    pub key_id: String,
    pub window_days: i64,
    pub visible: bool,
    pub status: BatchedRedactionBundleStatus,
}

impl BatchedRedactionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRedactionBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRedactionBundleStatus) -> bool {
        match self.status {
            BatchedRedactionBundleStatus::Open => {
                return next == BatchedRedactionBundleStatus::InReview;
            }
            BatchedRedactionBundleStatus::InReview => {
                return next == BatchedRedactionBundleStatus::Reopened
                    || next == BatchedRedactionBundleStatus::Archived;
            }
            BatchedRedactionBundleStatus::Resolved => {
                return next == BatchedRedactionBundleStatus::Archived;
            }
            BatchedRedactionBundleStatus::Reopened => {
                return next == BatchedRedactionBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRedactionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched redaction bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRedactionBundle {
        BatchedRedactionBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            port: self.port,
            created_at: self.created_at,
            key_id: self.key_id.clone(),
            window_days: self.window_days,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched redaction bundle in status open.
pub fn new_batched_redaction_bundle(code: &str, name: &str) -> BatchedRedactionBundle {
    BatchedRedactionBundle {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        port: 0,
        created_at: 0,
        key_id: "".to_string(),
        window_days: 0,
        visible: false,
        status: BatchedRedactionBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_redaction_bundle("code-73", "name-115");
        assert_eq!(item.status, BatchedRedactionBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_redaction_bundle("code-73", "name-115");
        if item.move_to(BatchedRedactionBundleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRedactionBundleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_redaction_bundle("code-73", "name-115");
        item.status = BatchedRedactionBundleStatus::Archived;
        assert!(
            item.move_to(BatchedRedactionBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
