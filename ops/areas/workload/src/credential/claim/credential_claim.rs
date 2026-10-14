use super::*;

use acme_kit::text;

/// Where a credential claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialClaimStatus {
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

impl std::fmt::Display for CredentialClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialClaim {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub owner_id: String,
    pub cost: f64,
    pub duration_minutes: i64,
    pub principal: String,
    pub revision: i64,
    pub status: CredentialClaimStatus,
}

impl CredentialClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialClaimStatus) -> bool {
        match self.status {
            CredentialClaimStatus::Open => {
                return next == CredentialClaimStatus::InReview;
            }
            CredentialClaimStatus::InReview => {
                return next == CredentialClaimStatus::Resolved;
            }
            CredentialClaimStatus::Resolved => {
                match next {
                    CredentialClaimStatus::Reopened | CredentialClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialClaimStatus::Reopened => {
                return next == CredentialClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential claim", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialClaim {
        CredentialClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential claim in status open.
pub fn new_credential_claim(code: &str, name: &str) -> CredentialClaim {
    CredentialClaim {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        duration_minutes: 0,
        principal: "".to_string(),
        revision: 0,
        status: CredentialClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_claim("code-117", "name-526");
        assert_eq!(item.status, CredentialClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_credential_claim("code-117", "name-526");
        if item.move_to(CredentialClaimStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialClaimStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_claim("code-117", "name-526");
        item.status = CredentialClaimStatus::Archived;
        assert!(
            item.move_to(CredentialClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
