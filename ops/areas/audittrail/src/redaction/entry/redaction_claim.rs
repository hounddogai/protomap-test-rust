use super::*;

use acme_kit::text;

/// Where a redaction claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RedactionClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RedactionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaim {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub audience: String,
    pub maximum: f64,
    pub status: RedactionClaimStatus,
}

impl RedactionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionClaimStatus) -> bool {
        match self.status {
            RedactionClaimStatus::Proposed => {
                return next == RedactionClaimStatus::Approved || next == RedactionClaimStatus::Rejected;
            }
            RedactionClaimStatus::Approved => {
                return next == RedactionClaimStatus::Withdrawn;
            }
            RedactionClaimStatus::Rejected => {
                return next == RedactionClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.margin + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionClaim {
        RedactionClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            audience: self.audience.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction claim in status proposed.
pub fn new_redaction_claim(code: &str, name: &str) -> RedactionClaim {
    RedactionClaim {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        audience: "".to_string(),
        maximum: 0.0,
        status: RedactionClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_redaction_claim("code-702", "name-780");
        assert_eq!(item.status, RedactionClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_redaction_claim("code-702", "name-780");
        if item.move_to(RedactionClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_redaction_claim("code-702", "name-780");
        item.status = RedactionClaimStatus::Withdrawn;
        assert!(
            item.move_to(RedactionClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
