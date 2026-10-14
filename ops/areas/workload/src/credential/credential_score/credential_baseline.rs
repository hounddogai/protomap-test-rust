use super::*;

use acme_kit::text;

/// Where a credential baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CredentialBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CredentialBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBaseline {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub reviewer: String,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub created_at: i64,
    pub status: CredentialBaselineStatus,
}

impl CredentialBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialBaselineStatus) -> bool {
        match self.status {
            CredentialBaselineStatus::Created => {
                return next == CredentialBaselineStatus::Validated || next == CredentialBaselineStatus::Published;
            }
            CredentialBaselineStatus::Validated => {
                return next == CredentialBaselineStatus::Published || next == CredentialBaselineStatus::Retired;
            }
            CredentialBaselineStatus::Published => {
                match next {
                    CredentialBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: CredentialBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential baseline {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialBaseline {
        CredentialBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            reviewer: self.reviewer.clone(),
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential baseline in status created.
pub fn new_credential_baseline(code: &str, title: &str) -> CredentialBaseline {
    CredentialBaseline {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        reviewer: "".to_string(),
        ttl_seconds: 0,
        margin: 0.0,
        created_at: 0,
        status: CredentialBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_credential_baseline("code-412", "title-638");
        assert_eq!(item.status, CredentialBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_credential_baseline("code-412", "title-638");
        if item.move_to(CredentialBaselineStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialBaselineStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_credential_baseline("code-412", "title-638");
        item.status = CredentialBaselineStatus::Retired;
        assert!(
            item.move_to(CredentialBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
