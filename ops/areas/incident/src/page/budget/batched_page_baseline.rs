use super::*;

use acme_kit::text;

/// Where a batched page baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPageBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedPageBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BatchedPageBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched page baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaseline {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub contact_email: String,
    pub rotation_days: i64,
    pub category: String,
    pub amount: f64,
    pub priority: i64,
    pub limit: i64,
    pub status: BatchedPageBaselineStatus,
}

impl BatchedPageBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPageBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPageBaselineStatus) -> bool {
        match self.status {
            BatchedPageBaselineStatus::Planned => {
                match next {
                    BatchedPageBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageBaselineStatus::Scheduled => {
                match next {
                    BatchedPageBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageBaselineStatus::InProgress => {
                return next == BatchedPageBaselineStatus::Completed;
            }
            BatchedPageBaselineStatus::Completed => {
                match next {
                    BatchedPageBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedPageBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched page baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched page baseline", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.priority as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPageBaseline {
        BatchedPageBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            contact_email: self.contact_email.clone(),
            rotation_days: self.rotation_days,
            category: self.category.clone(),
            amount: self.amount,
            priority: self.priority,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched page baseline in status planned.
pub fn new_batched_page_baseline(code: &str, name: &str) -> BatchedPageBaseline {
    BatchedPageBaseline {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        contact_email: "".to_string(),
        rotation_days: 0,
        category: "".to_string(),
        amount: 0.0,
        priority: 0,
        limit: 0,
        status: BatchedPageBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_page_baseline("code-56", "name-293");
        assert_eq!(item.status, BatchedPageBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_batched_page_baseline("code-56", "name-293");
        if item.move_to(BatchedPageBaselineStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPageBaselineStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_page_baseline("code-56", "name-293");
        item.status = BatchedPageBaselineStatus::Cancelled;
        assert!(
            item.move_to(BatchedPageBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
