use super::*;

use acme_kit::text;

/// Where a runbook variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookVarianceStatus {
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

impl std::fmt::Display for RunbookVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookVariance {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub audience: String,
    pub rate: f64,
    pub weight: f64,
    pub notes: String,
    pub margin: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub burst: i64,
    pub status: RunbookVarianceStatus,
}

impl RunbookVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookVarianceStatus) -> bool {
        match self.status {
            RunbookVarianceStatus::Planned => {
                return next == RunbookVarianceStatus::Completed || next == RunbookVarianceStatus::Cancelled;
            }
            RunbookVarianceStatus::Scheduled => {
                match next {
                    RunbookVarianceStatus::Completed | RunbookVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookVarianceStatus::InProgress => {
                match next {
                    RunbookVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookVarianceStatus::Completed => {
                return next == RunbookVarianceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.margin - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookVariance {
        RunbookVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            audience: self.audience.clone(),
            rate: self.rate,
            weight: self.weight,
            notes: self.notes.clone(),
            margin: self.margin,
            minimum: self.minimum,
            maximum: self.maximum,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a runbook variance in status planned.
pub fn new_runbook_variance(code: &str, title: &str) -> RunbookVariance {
    RunbookVariance {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        audience: "".to_string(),
        rate: 0.0,
        weight: 0.0,
        notes: "".to_string(),
        margin: 0.0,
        minimum: 0.0,
        maximum: 0.0,
        burst: 0,
        status: RunbookVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_runbook_variance("code-148", "title-972");
        assert_eq!(item.status, RunbookVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_runbook_variance("code-148", "title-972");
        if item.move_to(RunbookVarianceStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookVarianceStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_runbook_variance("code-148", "title-972");
        item.status = RunbookVarianceStatus::Cancelled;
        assert!(
            item.move_to(RunbookVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
