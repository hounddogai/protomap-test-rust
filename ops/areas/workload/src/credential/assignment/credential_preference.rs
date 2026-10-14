use super::*;

use acme_kit::text;

/// Where a credential preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CredentialPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialPreference {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub summary: String,
    pub scope: String,
    pub score: f64,
    pub status: CredentialPreferenceStatus,
}

impl CredentialPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialPreferenceStatus) -> bool {
        match self.status {
            CredentialPreferenceStatus::Idle => {
                match next {
                    CredentialPreferenceStatus::Warming | CredentialPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPreferenceStatus::Warming => {
                return next == CredentialPreferenceStatus::Draining || next == CredentialPreferenceStatus::Stopped;
            }
            CredentialPreferenceStatus::Ready => {
                match next {
                    CredentialPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPreferenceStatus::Draining => {
                return next == CredentialPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> CredentialPreference {
        CredentialPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a credential preference in status idle.
pub fn new_credential_preference(code: &str, label: &str) -> CredentialPreference {
    CredentialPreference {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        summary: "".to_string(),
        scope: "".to_string(),
        score: 0.0,
        status: CredentialPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_credential_preference("code-989", "label-316");
        assert_eq!(item.status, CredentialPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_credential_preference("code-989", "label-316");
        if item.move_to(CredentialPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_credential_preference("code-989", "label-316");
        item.status = CredentialPreferenceStatus::Stopped;
        assert!(
            item.move_to(CredentialPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
