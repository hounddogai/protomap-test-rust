use super::*;

use acme_kit::text;

/// Where a runbook agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RunbookAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RunbookAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreement {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub attempts: i64,
    pub version: i64,
    pub threshold: f64,
    pub locale: String,
    pub fingerprint: String,
    pub issuer: String,
    pub limit: i64,
    pub status: RunbookAgreementStatus,
}

impl RunbookAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookAgreementStatus) -> bool {
        match self.status {
            RunbookAgreementStatus::Draft => {
                return next == RunbookAgreementStatus::Pending || next == RunbookAgreementStatus::Suspended;
            }
            RunbookAgreementStatus::Pending => {
                match next {
                    RunbookAgreementStatus::Suspended | RunbookAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookAgreementStatus::Active => {
                return next == RunbookAgreementStatus::Suspended || next == RunbookAgreementStatus::Closed;
            }
            RunbookAgreementStatus::Suspended => {
                match next {
                    RunbookAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: RunbookAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook agreement", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.version as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookAgreement {
        RunbookAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            attempts: self.attempts,
            version: self.version,
            threshold: self.threshold,
            locale: self.locale.clone(),
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a runbook agreement in status draft.
pub fn new_runbook_agreement(code: &str, name: &str) -> RunbookAgreement {
    RunbookAgreement {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        attempts: 0,
        version: 0,
        threshold: 0.0,
        locale: "".to_string(),
        fingerprint: "".to_string(),
        issuer: "".to_string(),
        limit: 0,
        status: RunbookAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_runbook_agreement("code-113", "name-362");
        assert_eq!(item.status, RunbookAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_runbook_agreement("code-113", "name-362");
        if item.move_to(RunbookAgreementStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookAgreementStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_runbook_agreement("code-113", "name-362");
        item.status = RunbookAgreementStatus::Closed;
        assert!(
            item.move_to(RunbookAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
