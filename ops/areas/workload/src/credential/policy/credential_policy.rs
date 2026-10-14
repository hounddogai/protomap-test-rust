use super::*;

use acme_kit::text;

/// Where a credential policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CredentialPolicyStatus {
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

impl std::fmt::Display for CredentialPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialPolicy {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub updated_at: i64,
    pub segment: String,
    pub retries: i64,
    pub capacity: i64,
    pub issuer: String,
    pub revision: i64,
    pub status: CredentialPolicyStatus,
}

impl CredentialPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialPolicyStatus) -> bool {
        match self.status {
            CredentialPolicyStatus::Draft => {
                match next {
                    CredentialPolicyStatus::Active | CredentialPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPolicyStatus::Pending => {
                return next == CredentialPolicyStatus::Suspended || next == CredentialPolicyStatus::Closed;
            }
            CredentialPolicyStatus::Active => {
                match next {
                    CredentialPolicyStatus::Suspended | CredentialPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPolicyStatus::Suspended => {
                return next == CredentialPolicyStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential policy", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialPolicy {
        CredentialPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            segment: self.segment.clone(),
            retries: self.retries,
            capacity: self.capacity,
            issuer: self.issuer.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a credential policy in status draft.
pub fn new_credential_policy(code: &str, title: &str) -> CredentialPolicy {
    CredentialPolicy {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        updated_at: 0,
        segment: "".to_string(),
        retries: 0,
        capacity: 0,
        issuer: "".to_string(),
        revision: 0,
        status: CredentialPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_credential_policy("code-227", "title-238");
        assert_eq!(item.status, CredentialPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_credential_policy("code-227", "title-238");
        if item.move_to(CredentialPolicyStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialPolicyStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_credential_policy("code-227", "title-238");
        item.status = CredentialPolicyStatus::Closed;
        assert!(
            item.move_to(CredentialPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
