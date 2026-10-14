use super::*;

use acme_kit::text;

/// Where a scheduled silence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSilenceBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledSilenceBudgetStatus {
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

impl std::fmt::Display for ScheduledSilenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled silence budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSilenceBudget {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub approved: bool,
    pub tags: Vec<String>,
    pub status: ScheduledSilenceBudgetStatus,
}

impl ScheduledSilenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSilenceBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSilenceBudgetStatus) -> bool {
        match self.status {
            ScheduledSilenceBudgetStatus::Planned => {
                return next == ScheduledSilenceBudgetStatus::Scheduled
                    || next == ScheduledSilenceBudgetStatus::InProgress;
            }
            ScheduledSilenceBudgetStatus::Scheduled => {
                return next == ScheduledSilenceBudgetStatus::Cancelled;
            }
            ScheduledSilenceBudgetStatus::InProgress => {
                match next {
                    ScheduledSilenceBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSilenceBudgetStatus::Completed => {
                return next == ScheduledSilenceBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSilenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled silence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> ScheduledSilenceBudget {
        ScheduledSilenceBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            approved: self.approved,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled silence budget in status planned.
pub fn new_scheduled_silence_budget(code: &str, name: &str) -> ScheduledSilenceBudget {
    ScheduledSilenceBudget {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        approved: false,
        tags: Vec::new(),
        status: ScheduledSilenceBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_silence_budget("code-918", "name-875");
        assert_eq!(item.status, ScheduledSilenceBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_scheduled_silence_budget("code-918", "name-875");
        if item.move_to(ScheduledSilenceBudgetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSilenceBudgetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_silence_budget("code-918", "name-875");
        item.status = ScheduledSilenceBudgetStatus::Cancelled;
        assert!(
            item.move_to(ScheduledSilenceBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
