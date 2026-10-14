use super::*;

use acme_kit::text;

/// Where a rolling plan entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPlanEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingPlanEntryStatus {
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

impl std::fmt::Display for RollingPlanEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling plan entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanEntry {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub discount: f64,
    pub burst: i64,
    pub source: String,
    pub retries: i64,
    pub status: RollingPlanEntryStatus,
}

impl RollingPlanEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPlanEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPlanEntryStatus) -> bool {
        match self.status {
            RollingPlanEntryStatus::Planned => {
                match next {
                    RollingPlanEntryStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPlanEntryStatus::Scheduled => {
                return next == RollingPlanEntryStatus::Completed;
            }
            RollingPlanEntryStatus::InProgress => {
                return next == RollingPlanEntryStatus::Completed || next == RollingPlanEntryStatus::Cancelled;
            }
            RollingPlanEntryStatus::Completed => {
                match next {
                    RollingPlanEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingPlanEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling plan entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.discount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPlanEntry {
        RollingPlanEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            burst: self.burst,
            source: self.source.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling plan entry in status planned.
pub fn new_rolling_plan_entry(code: &str, name: &str) -> RollingPlanEntry {
    RollingPlanEntry {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        discount: 0.0,
        burst: 0,
        source: "".to_string(),
        retries: 0,
        status: RollingPlanEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_plan_entry("code-287", "name-711");
        assert_eq!(item.status, RollingPlanEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_plan_entry("code-287", "name-711");
        if item.move_to(RollingPlanEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPlanEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_plan_entry("code-287", "name-711");
        item.status = RollingPlanEntryStatus::Cancelled;
        assert!(
            item.move_to(RollingPlanEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
