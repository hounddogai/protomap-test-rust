use super::*;

use acme_kit::text;

/// Where a overage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl OverageBatchStatus {
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

impl std::fmt::Display for OverageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBatch {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub segment: String,
    pub pinned: bool,
    pub window_days: i64,
    pub phone: String,
    pub status: OverageBatchStatus,
}

impl OverageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageBatchStatus) -> bool {
        match self.status {
            OverageBatchStatus::Planned => {
                match next {
                    OverageBatchStatus::Scheduled | OverageBatchStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageBatchStatus::Scheduled => {
                return next == OverageBatchStatus::Completed;
            }
            OverageBatchStatus::InProgress => {
                match next {
                    OverageBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageBatchStatus::Completed => {
                match next {
                    OverageBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: OverageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageBatch {
        OverageBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            pinned: self.pinned,
            window_days: self.window_days,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a overage batch in status planned.
pub fn new_overage_batch(code: &str, title: &str) -> OverageBatch {
    OverageBatch {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        segment: "".to_string(),
        pinned: false,
        window_days: 0,
        phone: "".to_string(),
        status: OverageBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_overage_batch("code-606", "title-944");
        assert_eq!(item.status, OverageBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_overage_batch("code-606", "title-944");
        if item.move_to(OverageBatchStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageBatchStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_overage_batch("code-606", "title-944");
        item.status = OverageBatchStatus::Cancelled;
        assert!(
            item.move_to(OverageBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
