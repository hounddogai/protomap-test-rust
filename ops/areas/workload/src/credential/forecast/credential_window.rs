use super::*;

use acme_kit::text;

/// Where a credential window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialWindowStatus {
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

impl std::fmt::Display for CredentialWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential window as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialWindow {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub ratio: f64,
    pub duration_minutes: i64,
    pub rate: f64,
    pub region: String,
    pub retries: i64,
    pub scope: String,
    pub status: CredentialWindowStatus,
}

impl CredentialWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialWindowStatus) -> bool {
        match self.status {
            CredentialWindowStatus::Open => {
                return next == CredentialWindowStatus::Resolved;
            }
            CredentialWindowStatus::InReview => {
                return next == CredentialWindowStatus::Reopened || next == CredentialWindowStatus::Archived;
            }
            CredentialWindowStatus::Resolved => {
                match next {
                    CredentialWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialWindowStatus::Reopened => {
                return next == CredentialWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential window", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.rate * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialWindow {
        CredentialWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            ratio: self.ratio,
            duration_minutes: self.duration_minutes,
            rate: self.rate,
            region: self.region.clone(),
            retries: self.retries,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a credential window in status open.
pub fn new_credential_window(code: &str, name: &str) -> CredentialWindow {
    CredentialWindow {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        ratio: 0.0,
        duration_minutes: 0,
        rate: 0.0,
        region: "".to_string(),
        retries: 0,
        scope: "".to_string(),
        status: CredentialWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_window("code-197", "name-884");
        assert_eq!(item.status, CredentialWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_credential_window("code-197", "name-884");
        if item.move_to(CredentialWindowStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialWindowStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_window("code-197", "name-884");
        item.status = CredentialWindowStatus::Archived;
        assert!(
            item.move_to(CredentialWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
