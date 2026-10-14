use super::*;

use acme_kit::text;

/// Where a scheduled canary agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledCanaryAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledCanaryAgreementStatus {
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

impl std::fmt::Display for ScheduledCanaryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled canary agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryAgreement {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub sequence: i64,
    pub scope: String,
    pub status: ScheduledCanaryAgreementStatus,
}

impl ScheduledCanaryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledCanaryAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledCanaryAgreementStatus) -> bool {
        match self.status {
            ScheduledCanaryAgreementStatus::New => {
                match next {
                    ScheduledCanaryAgreementStatus::Queued | ScheduledCanaryAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCanaryAgreementStatus::Queued => {
                return next == ScheduledCanaryAgreementStatus::Done || next == ScheduledCanaryAgreementStatus::Failed;
            }
            ScheduledCanaryAgreementStatus::Running => {
                match next {
                    ScheduledCanaryAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCanaryAgreementStatus::Done => {
                return next == ScheduledCanaryAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledCanaryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled canary agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledCanaryAgreement {
        ScheduledCanaryAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            sequence: self.sequence,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled canary agreement in status new.
pub fn new_scheduled_canary_agreement(code: &str, title: &str) -> ScheduledCanaryAgreement {
    ScheduledCanaryAgreement {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        sequence: 0,
        scope: "".to_string(),
        status: ScheduledCanaryAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_canary_agreement("code-618", "title-44");
        assert_eq!(item.status, ScheduledCanaryAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_canary_agreement("code-618", "title-44");
        if item.move_to(ScheduledCanaryAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledCanaryAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_canary_agreement("code-618", "title-44");
        item.status = ScheduledCanaryAgreementStatus::Failed;
        assert!(
            item.move_to(ScheduledCanaryAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
