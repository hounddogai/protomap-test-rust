use super::*;

use acme_kit::text;

/// Where a retention cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RetentionCycleStatus {
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

impl std::fmt::Display for RetentionCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycle {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub title: String,
    pub summary: String,
    pub limit: i64,
    pub quantity: i64,
    pub namespace: String,
    pub status: RetentionCycleStatus,
}

impl RetentionCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionCycleStatus) -> bool {
        match self.status {
            RetentionCycleStatus::Planned => {
                match next {
                    RetentionCycleStatus::Completed | RetentionCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionCycleStatus::Scheduled => {
                match next {
                    RetentionCycleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionCycleStatus::InProgress => {
                return next == RetentionCycleStatus::Completed || next == RetentionCycleStatus::Cancelled;
            }
            RetentionCycleStatus::Completed => {
                return next == RetentionCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "retention cycle", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.limit as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionCycle {
        RetentionCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            title: self.title.clone(),
            summary: self.summary.clone(),
            limit: self.limit,
            quantity: self.quantity,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a retention cycle in status planned.
pub fn new_retention_cycle(code: &str, name: &str) -> RetentionCycle {
    RetentionCycle {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        title: "".to_string(),
        summary: "".to_string(),
        limit: 0,
        quantity: 0,
        namespace: "".to_string(),
        status: RetentionCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_retention_cycle("code-549", "name-330");
        assert_eq!(item.status, RetentionCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_retention_cycle("code-549", "name-330");
        if item.move_to(RetentionCycleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionCycleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_retention_cycle("code-549", "name-330");
        item.status = RetentionCycleStatus::Cancelled;
        assert!(
            item.move_to(RetentionCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
