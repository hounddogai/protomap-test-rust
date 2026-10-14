use super::*;

use acme_kit::text;

/// Where a batched query slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQuerySlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedQuerySlotStatus {
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

impl std::fmt::Display for BatchedQuerySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched query slot as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlot {
    pub code: String,
    pub title: String,
    pub host: String,
    pub locale: String,
    pub issuer: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub score: f64,
    pub ratio: f64,
    pub unit: String,
    pub fingerprint: String,
    pub status: BatchedQuerySlotStatus,
}

impl BatchedQuerySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQuerySlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQuerySlotStatus) -> bool {
        match self.status {
            BatchedQuerySlotStatus::New => {
                return next == BatchedQuerySlotStatus::Queued;
            }
            BatchedQuerySlotStatus::Queued => {
                match next {
                    BatchedQuerySlotStatus::Running | BatchedQuerySlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuerySlotStatus::Running => {
                match next {
                    BatchedQuerySlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuerySlotStatus::Done => {
                return next == BatchedQuerySlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedQuerySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched query slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ratio - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedQuerySlot {
        BatchedQuerySlot {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            score: self.score,
            ratio: self.ratio,
            unit: self.unit.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched query slot in status new.
pub fn new_batched_query_slot(code: &str, title: &str) -> BatchedQuerySlot {
    BatchedQuerySlot {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        locale: "".to_string(),
        issuer: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        score: 0.0,
        ratio: 0.0,
        unit: "".to_string(),
        fingerprint: "".to_string(),
        status: BatchedQuerySlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_query_slot("code-78", "title-361");
        assert_eq!(item.status, BatchedQuerySlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_query_slot("code-78", "title-361");
        if item.move_to(BatchedQuerySlotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQuerySlotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_query_slot("code-78", "title-361");
        item.status = BatchedQuerySlotStatus::Failed;
        assert!(
            item.move_to(BatchedQuerySlotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
