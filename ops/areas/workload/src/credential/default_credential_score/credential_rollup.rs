use super::*;

use acme_kit::text;

/// Where a credential rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential rollup as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollup {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub currency: String,
    pub status: CredentialRollupStatus,
}

impl CredentialRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialRollupStatus) -> bool {
        match self.status {
            CredentialRollupStatus::Requested => {
                match next {
                    CredentialRollupStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialRollupStatus::Granted => {
                return next == CredentialRollupStatus::Revoked || next == CredentialRollupStatus::Expired;
            }
            CredentialRollupStatus::Revoked => {
                return next == CredentialRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> CredentialRollup {
        CredentialRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a credential rollup in status requested.
pub fn new_credential_rollup(code: &str, label: &str) -> CredentialRollup {
    CredentialRollup {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        currency: "".to_string(),
        status: CredentialRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_rollup("code-816", "label-713");
        assert_eq!(item.status, CredentialRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_credential_rollup("code-816", "label-713");
        if item.move_to(CredentialRollupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialRollupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_rollup("code-816", "label-713");
        item.status = CredentialRollupStatus::Expired;
        assert!(
            item.move_to(CredentialRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
