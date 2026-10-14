use super::*;

use acme_kit::text;

/// Where a lenient rule quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientRuleQuotaStatus {
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

impl std::fmt::Display for LenientRuleQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient rule quota as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleQuota {
    pub code: String,
    pub name: String,
    pub host: String,
    pub region: String,
    pub discount: f64,
    pub status: LenientRuleQuotaStatus,
}

impl LenientRuleQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleQuotaStatus) -> bool {
        match self.status {
            LenientRuleQuotaStatus::Draft => {
                match next {
                    LenientRuleQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleQuotaStatus::Pending => {
                match next {
                    LenientRuleQuotaStatus::Active | LenientRuleQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleQuotaStatus::Active => {
                return next == LenientRuleQuotaStatus::Closed;
            }
            LenientRuleQuotaStatus::Suspended => {
                return next == LenientRuleQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRuleQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient rule quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> LenientRuleQuota {
        LenientRuleQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient rule quota in status draft.
pub fn new_lenient_rule_quota(code: &str, name: &str) -> LenientRuleQuota {
    LenientRuleQuota {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        region: "".to_string(),
        discount: 0.0,
        status: LenientRuleQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_rule_quota("code-847", "name-125");
        assert_eq!(item.status, LenientRuleQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_lenient_rule_quota("code-847", "name-125");
        if item.move_to(LenientRuleQuotaStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRuleQuotaStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_rule_quota("code-847", "name-125");
        item.status = LenientRuleQuotaStatus::Closed;
        assert!(
            item.move_to(LenientRuleQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
