use super::*;

use acme_kit::text;

/// Where a rollback summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackSummaryStatus {
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

impl std::fmt::Display for RollbackSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummary {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub namespace: String,
    pub threshold: f64,
    pub cost: f64,
    pub floor_value: f64,
    pub minimum: f64,
    pub cipher: String,
    pub scope: String,
    pub issuer: String,
    pub status: RollbackSummaryStatus,
}

impl RollbackSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackSummaryStatus) -> bool {
        match self.status {
            RollbackSummaryStatus::Planned => {
                return next == RollbackSummaryStatus::InProgress;
            }
            RollbackSummaryStatus::Scheduled => {
                return next == RollbackSummaryStatus::Completed;
            }
            RollbackSummaryStatus::InProgress => {
                match next {
                    RollbackSummaryStatus::Completed | RollbackSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSummaryStatus::Completed => {
                return next == RollbackSummaryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackSummary {
        RollbackSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            cost: self.cost,
            floor_value: self.floor_value,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rollback summary in status planned.
pub fn new_rollback_summary(code: &str, label: &str) -> RollbackSummary {
    RollbackSummary {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        namespace: "".to_string(),
        threshold: 0.0,
        cost: 0.0,
        floor_value: 0.0,
        minimum: 0.0,
        cipher: "".to_string(),
        scope: "".to_string(),
        issuer: "".to_string(),
        status: RollbackSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollback_summary("code-666", "label-509");
        assert_eq!(item.status, RollbackSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rollback_summary("code-666", "label-509");
        if item.move_to(RollbackSummaryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackSummaryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollback_summary("code-666", "label-509");
        item.status = RollbackSummaryStatus::Cancelled;
        assert!(
            item.move_to(RollbackSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
