use super::*;

use acme_kit::text;

/// Where a identity batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IdentityBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for IdentityBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity batch as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatch {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub host: String,
    pub amount: f64,
    pub status: IdentityBatchStatus,
}

impl IdentityBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityBatchStatus) -> bool {
        match self.status {
            IdentityBatchStatus::New => {
                match next {
                    IdentityBatchStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityBatchStatus::Queued => {
                return next == IdentityBatchStatus::Running || next == IdentityBatchStatus::Done;
            }
            IdentityBatchStatus::Running => {
                return next == IdentityBatchStatus::Done;
            }
            IdentityBatchStatus::Done => {
                match next {
                    IdentityBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: IdentityBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity batch", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.amount - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityBatch {
        IdentityBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity batch in status new.
pub fn new_identity_batch(code: &str, label: &str) -> IdentityBatch {
    IdentityBatch {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        tags: Vec::new(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        amount: 0.0,
        status: IdentityBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_identity_batch("code-628", "label-941");
        assert_eq!(item.status, IdentityBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_identity_batch("code-628", "label-941");
        if item.move_to(IdentityBatchStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityBatchStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_identity_batch("code-628", "label-941");
        item.status = IdentityBatchStatus::Failed;
        assert!(item.move_to(IdentityBatchStatus::New).is_err(), "expected move to fail");
    }
}
