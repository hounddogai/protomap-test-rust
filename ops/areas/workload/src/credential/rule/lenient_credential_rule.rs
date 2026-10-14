use super::*;

use acme_kit::text;

/// Where a lenient credential rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCredentialRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientCredentialRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientCredentialRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient credential rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialRule {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub source: String,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub approved: bool,
    pub status: LenientCredentialRuleStatus,
}

impl LenientCredentialRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCredentialRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCredentialRuleStatus) -> bool {
        match self.status {
            LenientCredentialRuleStatus::Proposed => {
                match next {
                    LenientCredentialRuleStatus::Approved | LenientCredentialRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCredentialRuleStatus::Approved => {
                return next == LenientCredentialRuleStatus::Rejected || next == LenientCredentialRuleStatus::Withdrawn;
            }
            LenientCredentialRuleStatus::Rejected => {
                return next == LenientCredentialRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientCredentialRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient credential rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCredentialRule {
        LenientCredentialRule {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient credential rule in status proposed.
pub fn new_lenient_credential_rule(code: &str, name: &str) -> LenientCredentialRule {
    LenientCredentialRule {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        source: "".to_string(),
        ttl_seconds: 0,
        floor_value: 0.0,
        approved: false,
        status: LenientCredentialRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_credential_rule("code-841", "name-124");
        assert_eq!(item.status, LenientCredentialRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_credential_rule("code-841", "name-124");
        if item.move_to(LenientCredentialRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCredentialRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_credential_rule("code-841", "name-124");
        item.status = LenientCredentialRuleStatus::Withdrawn;
        assert!(
            item.move_to(LenientCredentialRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
