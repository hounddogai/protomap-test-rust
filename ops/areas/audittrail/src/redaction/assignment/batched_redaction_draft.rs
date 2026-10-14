use super::*;

use acme_kit::text;

/// Where a batched redaction draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRedactionDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRedactionDraftStatus {
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

impl std::fmt::Display for BatchedRedactionDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched redaction draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraft {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub unit: String,
    pub contact_email: String,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub priority: i64,
    pub burst: i64,
    pub retries: i64,
    pub status: BatchedRedactionDraftStatus,
}

impl BatchedRedactionDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRedactionDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRedactionDraftStatus) -> bool {
        match self.status {
            BatchedRedactionDraftStatus::Open => {
                return next == BatchedRedactionDraftStatus::Reopened || next == BatchedRedactionDraftStatus::Archived;
            }
            BatchedRedactionDraftStatus::InReview => {
                return next == BatchedRedactionDraftStatus::Resolved || next == BatchedRedactionDraftStatus::Reopened;
            }
            BatchedRedactionDraftStatus::Resolved => {
                return next == BatchedRedactionDraftStatus::Reopened || next == BatchedRedactionDraftStatus::Archived;
            }
            BatchedRedactionDraftStatus::Reopened => {
                return next == BatchedRedactionDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRedactionDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched redaction draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRedactionDraft {
        BatchedRedactionDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            unit: self.unit.clone(),
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            priority: self.priority,
            burst: self.burst,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched redaction draft in status open.
pub fn new_batched_redaction_draft(code: &str, label: &str) -> BatchedRedactionDraft {
    BatchedRedactionDraft {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        unit: "".to_string(),
        contact_email: "".to_string(),
        starts_at: 0,
        tags: Vec::new(),
        priority: 0,
        burst: 0,
        retries: 0,
        status: BatchedRedactionDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_redaction_draft("code-425", "label-784");
        assert_eq!(item.status, BatchedRedactionDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_batched_redaction_draft("code-425", "label-784");
        if item.move_to(BatchedRedactionDraftStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRedactionDraftStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_redaction_draft("code-425", "label-784");
        item.status = BatchedRedactionDraftStatus::Archived;
        assert!(
            item.move_to(BatchedRedactionDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
