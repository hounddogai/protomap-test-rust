use super::*;

use acme_kit::text;

/// Where a global rollback agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRollbackAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalRollbackAgreementStatus {
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

impl std::fmt::Display for GlobalRollbackAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global rollback agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackAgreement {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub ceiling: f64,
    pub region: String,
    pub score: f64,
    pub cipher: String,
    pub port: i64,
    pub minimum: f64,
    pub description: String,
    pub locale: String,
    pub status: GlobalRollbackAgreementStatus,
}

impl GlobalRollbackAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRollbackAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRollbackAgreementStatus) -> bool {
        match self.status {
            GlobalRollbackAgreementStatus::New => {
                return next == GlobalRollbackAgreementStatus::Running;
            }
            GlobalRollbackAgreementStatus::Queued => {
                return next == GlobalRollbackAgreementStatus::Running || next == GlobalRollbackAgreementStatus::Done;
            }
            GlobalRollbackAgreementStatus::Running => {
                return next == GlobalRollbackAgreementStatus::Done || next == GlobalRollbackAgreementStatus::Failed;
            }
            GlobalRollbackAgreementStatus::Done => {
                match next {
                    GlobalRollbackAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalRollbackAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollback agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.score - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRollbackAgreement {
        GlobalRollbackAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            ceiling: self.ceiling,
            region: self.region.clone(),
            score: self.score,
            cipher: self.cipher.clone(),
            port: self.port,
            minimum: self.minimum,
            description: self.description.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global rollback agreement in status new.
pub fn new_global_rollback_agreement(code: &str, name: &str) -> GlobalRollbackAgreement {
    GlobalRollbackAgreement {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        ceiling: 0.0,
        region: "".to_string(),
        score: 0.0,
        cipher: "".to_string(),
        port: 0,
        minimum: 0.0,
        description: "".to_string(),
        locale: "".to_string(),
        status: GlobalRollbackAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_rollback_agreement("code-124", "name-783");
        assert_eq!(item.status, GlobalRollbackAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_global_rollback_agreement("code-124", "name-783");
        if item.move_to(GlobalRollbackAgreementStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRollbackAgreementStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_rollback_agreement("code-124", "name-783");
        item.status = GlobalRollbackAgreementStatus::Failed;
        assert!(
            item.move_to(GlobalRollbackAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
