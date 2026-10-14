use super::*;

use acme_kit::text;

/// Where a batched verifier bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedVerifierBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedVerifierBundleStatus {
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

impl std::fmt::Display for BatchedVerifierBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched verifier bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedVerifierBundle {
    pub code: String,
    pub title: String,
    pub name: String,
    pub currency: String,
    pub capacity: i64,
    pub tags: Vec<String>,
    pub priority: i64,
    pub status: BatchedVerifierBundleStatus,
}

impl BatchedVerifierBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedVerifierBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedVerifierBundleStatus) -> bool {
        match self.status {
            BatchedVerifierBundleStatus::New => {
                match next {
                    BatchedVerifierBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedVerifierBundleStatus::Queued => {
                return next == BatchedVerifierBundleStatus::Done || next == BatchedVerifierBundleStatus::Failed;
            }
            BatchedVerifierBundleStatus::Running => {
                match next {
                    BatchedVerifierBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedVerifierBundleStatus::Done => {
                match next {
                    BatchedVerifierBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedVerifierBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched verifier bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched verifier bundle", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.priority as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedVerifierBundle {
        BatchedVerifierBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            capacity: self.capacity,
            tags: self.tags.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched verifier bundle in status new.
pub fn new_batched_verifier_bundle(code: &str, title: &str) -> BatchedVerifierBundle {
    BatchedVerifierBundle {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        currency: "".to_string(),
        capacity: 0,
        tags: Vec::new(),
        priority: 0,
        status: BatchedVerifierBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_verifier_bundle("code-53", "title-613");
        assert_eq!(item.status, BatchedVerifierBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_batched_verifier_bundle("code-53", "title-613");
        if item.move_to(BatchedVerifierBundleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedVerifierBundleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_verifier_bundle("code-53", "title-613");
        item.status = BatchedVerifierBundleStatus::Failed;
        assert!(
            item.move_to(BatchedVerifierBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
