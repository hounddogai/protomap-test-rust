use super::*;

use acme_kit::text;

/// Where a tiered rule digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRuleDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredRuleDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredRuleDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered rule digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleDigest {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub tenant_id: String,
    pub quantity: i64,
    pub contact_email: String,
    pub amount: f64,
    pub version: i64,
    pub status: TieredRuleDigestStatus,
}

impl TieredRuleDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRuleDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRuleDigestStatus) -> bool {
        match self.status {
            TieredRuleDigestStatus::Proposed => {
                return next == TieredRuleDigestStatus::Approved || next == TieredRuleDigestStatus::Rejected;
            }
            TieredRuleDigestStatus::Approved => {
                match next {
                    TieredRuleDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRuleDigestStatus::Rejected => {
                match next {
                    TieredRuleDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredRuleDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rule digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.cost - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRuleDigest {
        TieredRuleDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered rule digest in status proposed.
pub fn new_tiered_rule_digest(code: &str, label: &str) -> TieredRuleDigest {
    TieredRuleDigest {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        tenant_id: "".to_string(),
        quantity: 0,
        contact_email: "".to_string(),
        amount: 0.0,
        version: 0,
        status: TieredRuleDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_rule_digest("code-159", "label-648");
        assert_eq!(item.status, TieredRuleDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_rule_digest("code-159", "label-648");
        if item.move_to(TieredRuleDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRuleDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_rule_digest("code-159", "label-648");
        item.status = TieredRuleDigestStatus::Withdrawn;
        assert!(
            item.move_to(TieredRuleDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
