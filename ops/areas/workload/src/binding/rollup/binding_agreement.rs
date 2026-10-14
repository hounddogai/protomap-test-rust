use super::*;

use acme_kit::text;

/// Where a binding agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BindingAgreementStatus {
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

impl std::fmt::Display for BindingAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAgreement {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub priority: i64,
    pub external_ref: String,
    pub namespace: String,
    pub limit: i64,
    pub retries: i64,
    pub checksum: String,
    pub channel: String,
    pub status: BindingAgreementStatus,
}

impl BindingAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingAgreementStatus) -> bool {
        match self.status {
            BindingAgreementStatus::New => {
                match next {
                    BindingAgreementStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingAgreementStatus::Queued => {
                return next == BindingAgreementStatus::Running;
            }
            BindingAgreementStatus::Running => {
                match next {
                    BindingAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingAgreementStatus::Done => {
                return next == BindingAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "binding agreement", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingAgreement {
        BindingAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            limit: self.limit,
            retries: self.retries,
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a binding agreement in status new.
pub fn new_binding_agreement(code: &str, name: &str) -> BindingAgreement {
    BindingAgreement {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        priority: 0,
        external_ref: "".to_string(),
        namespace: "".to_string(),
        limit: 0,
        retries: 0,
        checksum: "".to_string(),
        channel: "".to_string(),
        status: BindingAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_binding_agreement("code-98", "name-865");
        assert_eq!(item.status, BindingAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_binding_agreement("code-98", "name-865");
        if item.move_to(BindingAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_binding_agreement("code-98", "name-865");
        item.status = BindingAgreementStatus::Failed;
        assert!(
            item.move_to(BindingAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
