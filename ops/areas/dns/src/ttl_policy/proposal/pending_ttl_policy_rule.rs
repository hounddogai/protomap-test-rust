use super::*;

use acme_kit::text;

/// Where a pending ttl policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTtlPolicyRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingTtlPolicyRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingTtlPolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending ttl policy rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRule {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub active: bool,
    pub cipher: String,
    pub expires_at: i64,
    pub window_days: i64,
    pub quantity: i64,
    pub status: PendingTtlPolicyRuleStatus,
}

impl PendingTtlPolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTtlPolicyRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTtlPolicyRuleStatus) -> bool {
        match self.status {
            PendingTtlPolicyRuleStatus::Proposed => {
                return next == PendingTtlPolicyRuleStatus::Rejected || next == PendingTtlPolicyRuleStatus::Withdrawn;
            }
            PendingTtlPolicyRuleStatus::Approved => {
                return next == PendingTtlPolicyRuleStatus::Rejected || next == PendingTtlPolicyRuleStatus::Withdrawn;
            }
            PendingTtlPolicyRuleStatus::Rejected => {
                match next {
                    PendingTtlPolicyRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingTtlPolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending ttl policy rule {} cannot move from {} to {}",
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

    pub fn effective_total(&self) -> f64 {
        ((self.quantity as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTtlPolicyRule {
        PendingTtlPolicyRule {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            active: self.active,
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            window_days: self.window_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending ttl policy rule in status proposed.
pub fn new_pending_ttl_policy_rule(code: &str, name: &str) -> PendingTtlPolicyRule {
    PendingTtlPolicyRule {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        active: false,
        cipher: "".to_string(),
        expires_at: 0,
        window_days: 0,
        quantity: 0,
        status: PendingTtlPolicyRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_ttl_policy_rule("code-818", "name-732");
        assert_eq!(item.status, PendingTtlPolicyRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_pending_ttl_policy_rule("code-818", "name-732");
        if item.move_to(PendingTtlPolicyRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTtlPolicyRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_ttl_policy_rule("code-818", "name-732");
        item.status = PendingTtlPolicyRuleStatus::Withdrawn;
        assert!(
            item.move_to(PendingTtlPolicyRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
