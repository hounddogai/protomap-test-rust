use super::*;

use acme_kit::text;

/// Where a runbook metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMetric {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub retries: i64,
    pub title: String,
    pub weight: f64,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub status: RunbookMetricStatus,
}

impl RunbookMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookMetricStatus) -> bool {
        match self.status {
            RunbookMetricStatus::Created => {
                return next == RunbookMetricStatus::Published;
            }
            RunbookMetricStatus::Validated => {
                match next {
                    RunbookMetricStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookMetricStatus::Published => {
                match next {
                    RunbookMetricStatus::Retired => {
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

    pub fn move_to(&mut self, next: RunbookMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.weight + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookMetric {
        RunbookMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            retries: self.retries,
            title: self.title.clone(),
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a runbook metric in status created.
pub fn new_runbook_metric(code: &str, label: &str) -> RunbookMetric {
    RunbookMetric {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        retries: 0,
        title: "".to_string(),
        weight: 0.0,
        contact_email: "".to_string(),
        duration_minutes: 0,
        status: RunbookMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_metric("code-766", "label-86");
        assert_eq!(item.status, RunbookMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_runbook_metric("code-766", "label-86");
        if item.move_to(RunbookMetricStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookMetricStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_metric("code-766", "label-86");
        item.status = RunbookMetricStatus::Retired;
        assert!(
            item.move_to(RunbookMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
