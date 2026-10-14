use super::*;

use acme_kit::text;

/// Where a verifier claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaim {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub unit: String,
    pub priority: i64,
    pub ends_at: i64,
    pub active: bool,
    pub status: VerifierClaimStatus,
}

impl VerifierClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierClaimStatus) -> bool {
        match self.status {
            VerifierClaimStatus::Created => {
                return next == VerifierClaimStatus::Published || next == VerifierClaimStatus::Retired;
            }
            VerifierClaimStatus::Validated => {
                return next == VerifierClaimStatus::Retired;
            }
            VerifierClaimStatus::Published => {
                match next {
                    VerifierClaimStatus::Retired => {
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

    pub fn move_to(&mut self, next: VerifierClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ends_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierClaim {
        VerifierClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            unit: self.unit.clone(),
            priority: self.priority,
            ends_at: self.ends_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a verifier claim in status created.
pub fn new_verifier_claim(code: &str, title: &str) -> VerifierClaim {
    VerifierClaim {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        unit: "".to_string(),
        priority: 0,
        ends_at: 0,
        active: false,
        status: VerifierClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_verifier_claim("code-529", "title-461");
        assert_eq!(item.status, VerifierClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_verifier_claim("code-529", "title-461");
        if item.move_to(VerifierClaimStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierClaimStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_verifier_claim("code-529", "title-461");
        item.status = VerifierClaimStatus::Retired;
        assert!(
            item.move_to(VerifierClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
