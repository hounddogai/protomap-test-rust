use super::*;

use acme_kit::text;

/// Where a default timeline budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTimelineBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultTimelineBudgetStatus {
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

impl std::fmt::Display for DefaultTimelineBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default timeline budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTimelineBudget {
    pub code: String,
    pub label: String,
    pub title: String,
    pub timezone: String,
    pub ttl_seconds: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub weight: f64,
    pub status: DefaultTimelineBudgetStatus,
}

impl DefaultTimelineBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTimelineBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTimelineBudgetStatus) -> bool {
        match self.status {
            DefaultTimelineBudgetStatus::Planned => {
                return next == DefaultTimelineBudgetStatus::InProgress
                    || next == DefaultTimelineBudgetStatus::Cancelled;
            }
            DefaultTimelineBudgetStatus::Scheduled => {
                return next == DefaultTimelineBudgetStatus::InProgress
                    || next == DefaultTimelineBudgetStatus::Cancelled;
            }
            DefaultTimelineBudgetStatus::InProgress => {
                return next == DefaultTimelineBudgetStatus::Completed
                    || next == DefaultTimelineBudgetStatus::Cancelled;
            }
            DefaultTimelineBudgetStatus::Completed => {
                match next {
                    DefaultTimelineBudgetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultTimelineBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default timeline budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        (self.weight * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTimelineBudget {
        DefaultTimelineBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            ttl_seconds: self.ttl_seconds,
            burst: self.burst,
            quota_units: self.quota_units,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default timeline budget in status planned.
pub fn new_default_timeline_budget(code: &str, label: &str) -> DefaultTimelineBudget {
    DefaultTimelineBudget {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        timezone: "".to_string(),
        ttl_seconds: 0,
        burst: 0,
        quota_units: 0,
        weight: 0.0,
        status: DefaultTimelineBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_timeline_budget("code-782", "label-781");
        assert_eq!(item.status, DefaultTimelineBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_timeline_budget("code-782", "label-781");
        if item.move_to(DefaultTimelineBudgetStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTimelineBudgetStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_timeline_budget("code-782", "label-781");
        item.status = DefaultTimelineBudgetStatus::Cancelled;
        assert!(
            item.move_to(DefaultTimelineBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
