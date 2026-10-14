use super::*;

use acme_kit::text;

/// Where a escalation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EscalationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EscalationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationClaim {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub scope: String,
    pub attempts: i64,
    pub margin: f64,
    pub status: EscalationClaimStatus,
}

impl EscalationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationClaimStatus) -> bool {
        match self.status {
            EscalationClaimStatus::Created => {
                match next {
                    EscalationClaimStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationClaimStatus::Validated => {
                return next == EscalationClaimStatus::Published || next == EscalationClaimStatus::Retired;
            }
            EscalationClaimStatus::Published => {
                return next == EscalationClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation claim", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.margin + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationClaim {
        EscalationClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            scope: self.scope.clone(),
            attempts: self.attempts,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a escalation claim in status created.
pub fn new_escalation_claim(code: &str, title: &str) -> EscalationClaim {
    EscalationClaim {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        scope: "".to_string(),
        attempts: 0,
        margin: 0.0,
        status: EscalationClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_escalation_claim("code-867", "title-165");
        assert_eq!(item.status, EscalationClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_escalation_claim("code-867", "title-165");
        if item.move_to(EscalationClaimStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationClaimStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_escalation_claim("code-867", "title-165");
        item.status = EscalationClaimStatus::Retired;
        assert!(
            item.move_to(EscalationClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
