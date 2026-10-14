use super::*;

use acme_kit::text;

/// Where a delegation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRule {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub port: i64,
    pub maximum: f64,
    pub tags: Vec<String>,
    pub expires_at: i64,
    pub ends_at: i64,
    pub owner_id: String,
    pub contact_email: String,
    pub reviewer: String,
    pub status: DelegationRuleStatus,
}

impl DelegationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationRuleStatus) -> bool {
        match self.status {
            DelegationRuleStatus::Proposed => {
                return next == DelegationRuleStatus::Approved;
            }
            DelegationRuleStatus::Approved => {
                match next {
                    DelegationRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRuleStatus::Rejected => {
                return next == DelegationRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationRule {
        DelegationRule {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            port: self.port,
            maximum: self.maximum,
            tags: self.tags.clone(),
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation rule in status proposed.
pub fn new_delegation_rule(code: &str, label: &str) -> DelegationRule {
    DelegationRule {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        port: 0,
        maximum: 0.0,
        tags: Vec::new(),
        expires_at: 0,
        ends_at: 0,
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        reviewer: "".to_string(),
        status: DelegationRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_rule("code-157", "label-508");
        assert_eq!(item.status, DelegationRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_delegation_rule("code-157", "label-508");
        if item.move_to(DelegationRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_rule("code-157", "label-508");
        item.status = DelegationRuleStatus::Withdrawn;
        assert!(
            item.move_to(DelegationRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
