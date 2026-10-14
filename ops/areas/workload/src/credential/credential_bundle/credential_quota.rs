use super::*;

use acme_kit::text;

/// Where a credential quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CredentialQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuota {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub cipher: String,
    pub limit: i64,
    pub external_ref: String,
    pub capacity: i64,
    pub attempts: i64,
    pub created_at: i64,
    pub status: CredentialQuotaStatus,
}

impl CredentialQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialQuotaStatus) -> bool {
        match self.status {
            CredentialQuotaStatus::Planned => {
                return next == CredentialQuotaStatus::Cancelled;
            }
            CredentialQuotaStatus::Scheduled => {
                match next {
                    CredentialQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialQuotaStatus::InProgress => {
                return next == CredentialQuotaStatus::Completed;
            }
            CredentialQuotaStatus::Completed => {
                return next == CredentialQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential quota {} cannot move from {} to {}",
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
        format!("{} {}", "credential quota", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialQuota {
        CredentialQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            limit: self.limit,
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential quota in status planned.
pub fn new_credential_quota(code: &str, name: &str) -> CredentialQuota {
    CredentialQuota {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        cipher: "".to_string(),
        limit: 0,
        external_ref: "".to_string(),
        capacity: 0,
        attempts: 0,
        created_at: 0,
        status: CredentialQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_credential_quota("code-625", "name-332");
        assert_eq!(item.status, CredentialQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_credential_quota("code-625", "name-332");
        if item.move_to(CredentialQuotaStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialQuotaStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_credential_quota("code-625", "name-332");
        item.status = CredentialQuotaStatus::Cancelled;
        assert!(
            item.move_to(CredentialQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
