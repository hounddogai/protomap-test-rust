use super::*;

use acme_kit::text;

/// Where a global plan rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalPlanRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalPlanRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global plan rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanRule {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub locale: String,
    pub margin: f64,
    pub sequence: i64,
    pub rate: f64,
    pub maximum: f64,
    pub category: String,
    pub issuer: String,
    pub status: GlobalPlanRuleStatus,
}

impl GlobalPlanRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanRuleStatus) -> bool {
        match self.status {
            GlobalPlanRuleStatus::Requested => {
                return next == GlobalPlanRuleStatus::Expired;
            }
            GlobalPlanRuleStatus::Granted => {
                match next {
                    GlobalPlanRuleStatus::Revoked | GlobalPlanRuleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanRuleStatus::Revoked => {
                return next == GlobalPlanRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPlanRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan rule {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ends_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanRule {
        GlobalPlanRule {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            locale: self.locale.clone(),
            margin: self.margin,
            sequence: self.sequence,
            rate: self.rate,
            maximum: self.maximum,
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global plan rule in status requested.
pub fn new_global_plan_rule(code: &str, name: &str) -> GlobalPlanRule {
    GlobalPlanRule {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        locale: "".to_string(),
        margin: 0.0,
        sequence: 0,
        rate: 0.0,
        maximum: 0.0,
        category: "".to_string(),
        issuer: "".to_string(),
        status: GlobalPlanRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_plan_rule("code-278", "name-372");
        assert_eq!(item.status, GlobalPlanRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_global_plan_rule("code-278", "name-372");
        if item.move_to(GlobalPlanRuleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPlanRuleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_plan_rule("code-278", "name-372");
        item.status = GlobalPlanRuleStatus::Expired;
        assert!(
            item.move_to(GlobalPlanRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
