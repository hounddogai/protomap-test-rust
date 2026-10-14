use super::*;

use acme_kit::text;

/// Where a manual health check score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualHealthCheckScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualHealthCheckScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualHealthCheckScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual health check score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScore {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub floor_value: f64,
    pub revision: i64,
    pub status: ManualHealthCheckScoreStatus,
}

impl ManualHealthCheckScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualHealthCheckScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualHealthCheckScoreStatus) -> bool {
        match self.status {
            ManualHealthCheckScoreStatus::New => {
                match next {
                    ManualHealthCheckScoreStatus::Queued | ManualHealthCheckScoreStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualHealthCheckScoreStatus::Queued => {
                return next == ManualHealthCheckScoreStatus::Running;
            }
            ManualHealthCheckScoreStatus::Running => {
                return next == ManualHealthCheckScoreStatus::Failed;
            }
            ManualHealthCheckScoreStatus::Done => {
                return next == ManualHealthCheckScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualHealthCheckScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual health check score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.revision as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualHealthCheckScore {
        ManualHealthCheckScore {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual health check score in status new.
pub fn new_manual_health_check_score(code: &str, title: &str) -> ManualHealthCheckScore {
    ManualHealthCheckScore {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        floor_value: 0.0,
        revision: 0,
        status: ManualHealthCheckScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_health_check_score("code-447", "title-819");
        assert_eq!(item.status, ManualHealthCheckScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_health_check_score("code-447", "title-819");
        if item.move_to(ManualHealthCheckScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualHealthCheckScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_health_check_score("code-447", "title-819");
        item.status = ManualHealthCheckScoreStatus::Failed;
        assert!(
            item.move_to(ManualHealthCheckScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
