use super::*;

use acme_kit::text;

/// Where a credential exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CredentialExceptionStatus {
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

impl std::fmt::Display for CredentialExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential exception as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialException {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub priority: i64,
    pub visible: bool,
    pub cluster: String,
    pub unit: String,
    pub attempts: i64,
    pub minimum: f64,
    pub status: CredentialExceptionStatus,
}

impl CredentialException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialExceptionStatus) -> bool {
        match self.status {
            CredentialExceptionStatus::New => {
                match next {
                    CredentialExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialExceptionStatus::Queued => {
                return next == CredentialExceptionStatus::Running || next == CredentialExceptionStatus::Failed;
            }
            CredentialExceptionStatus::Running => {
                match next {
                    CredentialExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialExceptionStatus::Done => {
                match next {
                    CredentialExceptionStatus::Failed => {
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

    pub fn move_to(&mut self, next: CredentialExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential exception", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.priority as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialException {
        CredentialException {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            priority: self.priority,
            visible: self.visible,
            cluster: self.cluster.clone(),
            unit: self.unit.clone(),
            attempts: self.attempts,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a credential exception in status new.
pub fn new_credential_exception(code: &str, name: &str) -> CredentialException {
    CredentialException {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        priority: 0,
        visible: false,
        cluster: "".to_string(),
        unit: "".to_string(),
        attempts: 0,
        minimum: 0.0,
        status: CredentialExceptionStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_credential_exception("code-989", "name-103");
        assert_eq!(item.status, CredentialExceptionStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_credential_exception("code-989", "name-103");
        if item.move_to(CredentialExceptionStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialExceptionStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_credential_exception("code-989", "name-103");
        item.status = CredentialExceptionStatus::Failed;
        assert!(
            item.move_to(CredentialExceptionStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
