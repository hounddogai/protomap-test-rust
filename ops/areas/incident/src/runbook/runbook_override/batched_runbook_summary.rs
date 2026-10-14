use super::*;

use acme_kit::text;

/// Where a batched runbook summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRunbookSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedRunbookSummaryStatus {
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

impl std::fmt::Display for BatchedRunbookSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched runbook summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummary {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub approved: bool,
    pub key_id: String,
    pub maximum: f64,
    pub status: BatchedRunbookSummaryStatus,
}

impl BatchedRunbookSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRunbookSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRunbookSummaryStatus) -> bool {
        match self.status {
            BatchedRunbookSummaryStatus::Draft => {
                return next == BatchedRunbookSummaryStatus::Suspended || next == BatchedRunbookSummaryStatus::Closed;
            }
            BatchedRunbookSummaryStatus::Pending => {
                return next == BatchedRunbookSummaryStatus::Active || next == BatchedRunbookSummaryStatus::Closed;
            }
            BatchedRunbookSummaryStatus::Active => {
                match next {
                    BatchedRunbookSummaryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRunbookSummaryStatus::Suspended => {
                match next {
                    BatchedRunbookSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedRunbookSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched runbook summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ceiling + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRunbookSummary {
        BatchedRunbookSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched runbook summary in status draft.
pub fn new_batched_runbook_summary(code: &str, name: &str) -> BatchedRunbookSummary {
    BatchedRunbookSummary {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        approved: false,
        key_id: "".to_string(),
        maximum: 0.0,
        status: BatchedRunbookSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_runbook_summary("code-723", "name-908");
        assert_eq!(item.status, BatchedRunbookSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_runbook_summary("code-723", "name-908");
        if item.move_to(BatchedRunbookSummaryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRunbookSummaryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_runbook_summary("code-723", "name-908");
        item.status = BatchedRunbookSummaryStatus::Closed;
        assert!(
            item.move_to(BatchedRunbookSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
