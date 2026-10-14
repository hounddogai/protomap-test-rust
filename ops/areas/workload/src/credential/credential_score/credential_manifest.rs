use super::*;

use acme_kit::text;

/// Where a credential manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CredentialManifestStatus {
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

impl std::fmt::Display for CredentialManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential manifest as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialManifest {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub checksum: String,
    pub amount: f64,
    pub capacity: i64,
    pub approved: bool,
    pub reviewer: String,
    pub status: CredentialManifestStatus,
}

impl CredentialManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialManifestStatus) -> bool {
        match self.status {
            CredentialManifestStatus::New => {
                match next {
                    CredentialManifestStatus::Running | CredentialManifestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialManifestStatus::Queued => {
                match next {
                    CredentialManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialManifestStatus::Running => {
                match next {
                    CredentialManifestStatus::Done | CredentialManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialManifestStatus::Done => {
                match next {
                    CredentialManifestStatus::Failed => {
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

    pub fn move_to(&mut self, next: CredentialManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential manifest", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.capacity as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialManifest {
        CredentialManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            capacity: self.capacity,
            approved: self.approved,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential manifest in status new.
pub fn new_credential_manifest(code: &str, name: &str) -> CredentialManifest {
    CredentialManifest {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        capacity: 0,
        approved: false,
        reviewer: "".to_string(),
        status: CredentialManifestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_credential_manifest("code-137", "name-56");
        assert_eq!(item.status, CredentialManifestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_credential_manifest("code-137", "name-56");
        if item.move_to(CredentialManifestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialManifestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_credential_manifest("code-137", "name-56");
        item.status = CredentialManifestStatus::Failed;
        assert!(
            item.move_to(CredentialManifestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
