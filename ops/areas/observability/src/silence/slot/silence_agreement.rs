use super::*;

use acme_kit::text;

/// Where a silence agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SilenceAgreementStatus {
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

impl std::fmt::Display for SilenceAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreement {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub scope: String,
    pub created_at: i64,
    pub sequence: i64,
    pub revision: i64,
    pub port: i64,
    pub contact_email: String,
    pub priority: i64,
    pub ceiling: f64,
    pub status: SilenceAgreementStatus,
}

impl SilenceAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceAgreementStatus) -> bool {
        match self.status {
            SilenceAgreementStatus::Planned => {
                return next == SilenceAgreementStatus::Completed;
            }
            SilenceAgreementStatus::Scheduled => {
                return next == SilenceAgreementStatus::Cancelled;
            }
            SilenceAgreementStatus::InProgress => {
                return next == SilenceAgreementStatus::Cancelled;
            }
            SilenceAgreementStatus::Completed => {
                match next {
                    SilenceAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SilenceAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence agreement", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceAgreement {
        SilenceAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            scope: self.scope.clone(),
            created_at: self.created_at,
            sequence: self.sequence,
            revision: self.revision,
            port: self.port,
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a silence agreement in status planned.
pub fn new_silence_agreement(code: &str, name: &str) -> SilenceAgreement {
    SilenceAgreement {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        scope: "".to_string(),
        created_at: 0,
        sequence: 0,
        revision: 0,
        port: 0,
        contact_email: "".to_string(),
        priority: 0,
        ceiling: 0.0,
        status: SilenceAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_silence_agreement("code-361", "name-976");
        assert_eq!(item.status, SilenceAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_silence_agreement("code-361", "name-976");
        if item.move_to(SilenceAgreementStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceAgreementStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_silence_agreement("code-361", "name-976");
        item.status = SilenceAgreementStatus::Cancelled;
        assert!(
            item.move_to(SilenceAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
