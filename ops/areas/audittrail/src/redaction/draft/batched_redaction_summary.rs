use super::*;

use acme_kit::text;

/// Where a batched redaction summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRedactionSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedRedactionSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedRedactionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched redaction summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummary {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub key_id: String,
    pub approved: bool,
    pub sequence: i64,
    pub category: String,
    pub status: BatchedRedactionSummaryStatus,
}

impl BatchedRedactionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRedactionSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRedactionSummaryStatus) -> bool {
        match self.status {
            BatchedRedactionSummaryStatus::Requested => {
                return next == BatchedRedactionSummaryStatus::Granted
                    || next == BatchedRedactionSummaryStatus::Expired;
            }
            BatchedRedactionSummaryStatus::Granted => {
                return next == BatchedRedactionSummaryStatus::Revoked
                    || next == BatchedRedactionSummaryStatus::Expired;
            }
            BatchedRedactionSummaryStatus::Revoked => {
                return next == BatchedRedactionSummaryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRedactionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched redaction summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRedactionSummary {
        BatchedRedactionSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            approved: self.approved,
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched redaction summary in status requested.
pub fn new_batched_redaction_summary(code: &str, label: &str) -> BatchedRedactionSummary {
    BatchedRedactionSummary {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        key_id: "".to_string(),
        approved: false,
        sequence: 0,
        category: "".to_string(),
        status: BatchedRedactionSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_redaction_summary("code-58", "label-111");
        assert_eq!(item.status, BatchedRedactionSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_redaction_summary("code-58", "label-111");
        if item.move_to(BatchedRedactionSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRedactionSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_redaction_summary("code-58", "label-111");
        item.status = BatchedRedactionSummaryStatus::Expired;
        assert!(
            item.move_to(BatchedRedactionSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
