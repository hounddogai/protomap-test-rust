use super::*;

use acme_kit::text;

/// Where a pending escalation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingEscalationClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingEscalationClaimStatus {
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

impl std::fmt::Display for PendingEscalationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending escalation claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEscalationClaim {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub sequence: i64,
    pub tags: Vec<String>,
    pub issuer: String,
    pub status: PendingEscalationClaimStatus,
}

impl PendingEscalationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingEscalationClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingEscalationClaimStatus) -> bool {
        match self.status {
            PendingEscalationClaimStatus::Draft => {
                match next {
                    PendingEscalationClaimStatus::Active | PendingEscalationClaimStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationClaimStatus::Pending => {
                match next {
                    PendingEscalationClaimStatus::Active | PendingEscalationClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationClaimStatus::Active => {
                match next {
                    PendingEscalationClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEscalationClaimStatus::Suspended => {
                return next == PendingEscalationClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingEscalationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending escalation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.budget - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingEscalationClaim {
        PendingEscalationClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            sequence: self.sequence,
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending escalation claim in status draft.
pub fn new_pending_escalation_claim(code: &str, name: &str) -> PendingEscalationClaim {
    PendingEscalationClaim {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        sequence: 0,
        tags: Vec::new(),
        issuer: "".to_string(),
        status: PendingEscalationClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_escalation_claim("code-379", "name-5");
        assert_eq!(item.status, PendingEscalationClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_escalation_claim("code-379", "name-5");
        if item.move_to(PendingEscalationClaimStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingEscalationClaimStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_escalation_claim("code-379", "name-5");
        item.status = PendingEscalationClaimStatus::Closed;
        assert!(
            item.move_to(PendingEscalationClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
