use super::*;

use acme_kit::text;

/// Where a tiered escalation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredEscalationAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredEscalationAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredEscalationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered escalation agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationAgreement {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub principal: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub status: TieredEscalationAgreementStatus,
}

impl TieredEscalationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredEscalationAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredEscalationAgreementStatus) -> bool {
        match self.status {
            TieredEscalationAgreementStatus::Proposed => {
                return next == TieredEscalationAgreementStatus::Withdrawn;
            }
            TieredEscalationAgreementStatus::Approved => {
                match next {
                    TieredEscalationAgreementStatus::Rejected | TieredEscalationAgreementStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredEscalationAgreementStatus::Rejected => {
                return next == TieredEscalationAgreementStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredEscalationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered escalation agreement {} cannot move from {} to {}",
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

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> TieredEscalationAgreement {
        TieredEscalationAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            principal: self.principal.clone(),
            tags: self.tags.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered escalation agreement in status proposed.
pub fn new_tiered_escalation_agreement(code: &str, name: &str) -> TieredEscalationAgreement {
    TieredEscalationAgreement {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        principal: "".to_string(),
        tags: Vec::new(),
        active: false,
        status: TieredEscalationAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_escalation_agreement("code-448", "name-276");
        assert_eq!(item.status, TieredEscalationAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_tiered_escalation_agreement("code-448", "name-276");
        if item.move_to(TieredEscalationAgreementStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredEscalationAgreementStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_escalation_agreement("code-448", "name-276");
        item.status = TieredEscalationAgreementStatus::Withdrawn;
        assert!(
            item.move_to(TieredEscalationAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
