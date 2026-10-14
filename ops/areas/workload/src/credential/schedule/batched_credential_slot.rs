use super::*;

use acme_kit::text;

/// Where a batched credential slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedCredentialSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedCredentialSlotStatus {
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

impl std::fmt::Display for BatchedCredentialSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched credential slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialSlot {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub score: f64,
    pub segment: String,
    pub retries: i64,
    pub title: String,
    pub pinned: bool,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub scope: String,
    pub status: BatchedCredentialSlotStatus,
}

impl BatchedCredentialSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedCredentialSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedCredentialSlotStatus) -> bool {
        match self.status {
            BatchedCredentialSlotStatus::Open => {
                return next == BatchedCredentialSlotStatus::InReview || next == BatchedCredentialSlotStatus::Reopened;
            }
            BatchedCredentialSlotStatus::InReview => {
                match next {
                    BatchedCredentialSlotStatus::Resolved | BatchedCredentialSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCredentialSlotStatus::Resolved => {
                return next == BatchedCredentialSlotStatus::Reopened;
            }
            BatchedCredentialSlotStatus::Reopened => {
                return next == BatchedCredentialSlotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedCredentialSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched credential slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedCredentialSlot {
        BatchedCredentialSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            score: self.score,
            segment: self.segment.clone(),
            retries: self.retries,
            title: self.title.clone(),
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a batched credential slot in status open.
pub fn new_batched_credential_slot(code: &str, name: &str) -> BatchedCredentialSlot {
    BatchedCredentialSlot {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        score: 0.0,
        segment: "".to_string(),
        retries: 0,
        title: "".to_string(),
        pinned: false,
        cluster: "".to_string(),
        ttl_seconds: 0,
        scope: "".to_string(),
        status: BatchedCredentialSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_credential_slot("code-289", "name-261");
        assert_eq!(item.status, BatchedCredentialSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_batched_credential_slot("code-289", "name-261");
        if item.move_to(BatchedCredentialSlotStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedCredentialSlotStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_credential_slot("code-289", "name-261");
        item.status = BatchedCredentialSlotStatus::Archived;
        assert!(
            item.move_to(BatchedCredentialSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
