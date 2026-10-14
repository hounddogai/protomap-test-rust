use super::*;

use acme_kit::text;

/// Where a credential checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CredentialCheckpointStatus {
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

impl std::fmt::Display for CredentialCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential checkpoint as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCheckpoint {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub audience: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub fingerprint: String,
    pub status: CredentialCheckpointStatus,
}

impl CredentialCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialCheckpointStatus) -> bool {
        match self.status {
            CredentialCheckpointStatus::Draft => {
                return next == CredentialCheckpointStatus::Closed;
            }
            CredentialCheckpointStatus::Pending => {
                return next == CredentialCheckpointStatus::Active || next == CredentialCheckpointStatus::Suspended;
            }
            CredentialCheckpointStatus::Active => {
                match next {
                    CredentialCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialCheckpointStatus::Suspended => {
                match next {
                    CredentialCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: CredentialCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ends_at as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialCheckpoint {
        CredentialCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            audience: self.audience.clone(),
            created_at: self.created_at,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential checkpoint in status draft.
pub fn new_credential_checkpoint(code: &str, name: &str) -> CredentialCheckpoint {
    CredentialCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        audience: "".to_string(),
        created_at: 0,
        ends_at: 0,
        fingerprint: "".to_string(),
        status: CredentialCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_credential_checkpoint("code-90", "name-869");
        assert_eq!(item.status, CredentialCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_credential_checkpoint("code-90", "name-869");
        if item.move_to(CredentialCheckpointStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialCheckpointStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_credential_checkpoint("code-90", "name-869");
        item.status = CredentialCheckpointStatus::Closed;
        assert!(
            item.move_to(CredentialCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
