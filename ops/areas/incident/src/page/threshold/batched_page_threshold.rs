use super::*;

use acme_kit::text;

/// Where a batched page threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPageThresholdStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedPageThresholdStatus {
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

impl std::fmt::Display for BatchedPageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched page threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThreshold {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub discount: f64,
    pub pinned: bool,
    pub ceiling: f64,
    pub category: String,
    pub namespace: String,
    pub reviewer: String,
    pub limit: i64,
    pub status: BatchedPageThresholdStatus,
}

impl BatchedPageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPageThresholdStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPageThresholdStatus) -> bool {
        match self.status {
            BatchedPageThresholdStatus::Planned => {
                match next {
                    BatchedPageThresholdStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageThresholdStatus::Scheduled => {
                return next == BatchedPageThresholdStatus::Completed || next == BatchedPageThresholdStatus::Cancelled;
            }
            BatchedPageThresholdStatus::InProgress => {
                match next {
                    BatchedPageThresholdStatus::Completed | BatchedPageThresholdStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPageThresholdStatus::Completed => {
                return next == BatchedPageThresholdStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched page threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.discount - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPageThreshold {
        BatchedPageThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            discount: self.discount,
            pinned: self.pinned,
            ceiling: self.ceiling,
            category: self.category.clone(),
            namespace: self.namespace.clone(),
            reviewer: self.reviewer.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched page threshold in status planned.
pub fn new_batched_page_threshold(code: &str, name: &str) -> BatchedPageThreshold {
    BatchedPageThreshold {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        discount: 0.0,
        pinned: false,
        ceiling: 0.0,
        category: "".to_string(),
        namespace: "".to_string(),
        reviewer: "".to_string(),
        limit: 0,
        status: BatchedPageThresholdStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_page_threshold("code-437", "name-46");
        assert_eq!(item.status, BatchedPageThresholdStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_batched_page_threshold("code-437", "name-46");
        if item.move_to(BatchedPageThresholdStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPageThresholdStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_page_threshold("code-437", "name-46");
        item.status = BatchedPageThresholdStatus::Cancelled;
        assert!(
            item.move_to(BatchedPageThresholdStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
