use super::*;

use acme_kit::text;

/// Where a credential tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialTierStatus {
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

impl std::fmt::Display for CredentialTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTier {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub attempts: i64,
    pub summary: String,
    pub cipher: String,
    pub expires_at: i64,
    pub notes: String,
    pub phone: String,
    pub currency: String,
    pub status: CredentialTierStatus,
}

impl CredentialTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialTierStatus) -> bool {
        match self.status {
            CredentialTierStatus::Idle => {
                return next == CredentialTierStatus::Stopped;
            }
            CredentialTierStatus::Warming => {
                match next {
                    CredentialTierStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialTierStatus::Ready => {
                return next == CredentialTierStatus::Draining || next == CredentialTierStatus::Stopped;
            }
            CredentialTierStatus::Draining => {
                match next {
                    CredentialTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CredentialTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialTier {
        CredentialTier {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            attempts: self.attempts,
            summary: self.summary.clone(),
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            notes: self.notes.clone(),
            phone: self.phone.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a credential tier in status idle.
pub fn new_credential_tier(code: &str, name: &str) -> CredentialTier {
    CredentialTier {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        attempts: 0,
        summary: "".to_string(),
        cipher: "".to_string(),
        expires_at: 0,
        notes: "".to_string(),
        phone: "".to_string(),
        currency: "".to_string(),
        status: CredentialTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_credential_tier("code-761", "name-533");
        assert_eq!(item.status, CredentialTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_credential_tier("code-761", "name-533");
        if item.move_to(CredentialTierStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialTierStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_credential_tier("code-761", "name-533");
        item.status = CredentialTierStatus::Stopped;
        assert!(
            item.move_to(CredentialTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
