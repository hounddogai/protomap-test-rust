use super::*;

use acme_kit::text;

/// Where a runbook estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RunbookEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RunbookEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook estimate as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimate {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub version: i64,
    pub phone: String,
    pub key_id: String,
    pub grace_days: i64,
    pub reviewer: String,
    pub rate: f64,
    pub floor_value: f64,
    pub status: RunbookEstimateStatus,
}

impl RunbookEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookEstimateStatus) -> bool {
        match self.status {
            RunbookEstimateStatus::Open => {
                match next {
                    RunbookEstimateStatus::InReview | RunbookEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookEstimateStatus::InReview => {
                return next == RunbookEstimateStatus::Resolved || next == RunbookEstimateStatus::Archived;
            }
            RunbookEstimateStatus::Resolved => {
                match next {
                    RunbookEstimateStatus::Reopened | RunbookEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookEstimateStatus::Reopened => {
                match next {
                    RunbookEstimateStatus::Archived => {
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

    pub fn move_to(&mut self, next: RunbookEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookEstimate {
        RunbookEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            version: self.version,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            rate: self.rate,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a runbook estimate in status open.
pub fn new_runbook_estimate(code: &str, title: &str) -> RunbookEstimate {
    RunbookEstimate {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        version: 0,
        phone: "".to_string(),
        key_id: "".to_string(),
        grace_days: 0,
        reviewer: "".to_string(),
        rate: 0.0,
        floor_value: 0.0,
        status: RunbookEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_runbook_estimate("code-121", "title-234");
        assert_eq!(item.status, RunbookEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_runbook_estimate("code-121", "title-234");
        if item.move_to(RunbookEstimateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookEstimateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_runbook_estimate("code-121", "title-234");
        item.status = RunbookEstimateStatus::Archived;
        assert!(
            item.move_to(RunbookEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
