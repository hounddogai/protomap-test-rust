use super::*;

use acme_kit::text;

/// Where a rollout agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RolloutAgreementStatus {
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

impl std::fmt::Display for RolloutAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout agreement as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreement {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub starts_at: i64,
    pub archived: bool,
    pub maximum: f64,
    pub revision: i64,
    pub issuer: String,
    pub status: RolloutAgreementStatus,
}

impl RolloutAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutAgreementStatus) -> bool {
        match self.status {
            RolloutAgreementStatus::Planned => {
                match next {
                    RolloutAgreementStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutAgreementStatus::Scheduled => {
                match next {
                    RolloutAgreementStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutAgreementStatus::InProgress => {
                return next == RolloutAgreementStatus::Cancelled;
            }
            RolloutAgreementStatus::Completed => {
                return next == RolloutAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutAgreement {
        RolloutAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            archived: self.archived,
            maximum: self.maximum,
            revision: self.revision,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout agreement in status planned.
pub fn new_rollout_agreement(code: &str, title: &str) -> RolloutAgreement {
    RolloutAgreement {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        starts_at: 0,
        archived: false,
        maximum: 0.0,
        revision: 0,
        issuer: "".to_string(),
        status: RolloutAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollout_agreement("code-601", "title-477");
        assert_eq!(item.status, RolloutAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollout_agreement("code-601", "title-477");
        if item.move_to(RolloutAgreementStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutAgreementStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollout_agreement("code-601", "title-477");
        item.status = RolloutAgreementStatus::Cancelled;
        assert!(
            item.move_to(RolloutAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
