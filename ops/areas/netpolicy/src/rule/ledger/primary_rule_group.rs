use super::*;

use acme_kit::text;

/// Where a primary rule group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRuleGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRuleGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rule group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleGroup {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub region: String,
    pub category: String,
    pub channel: String,
    pub status: PrimaryRuleGroupStatus,
}

impl PrimaryRuleGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleGroupStatus) -> bool {
        match self.status {
            PrimaryRuleGroupStatus::Requested => {
                match next {
                    PrimaryRuleGroupStatus::Granted | PrimaryRuleGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleGroupStatus::Granted => {
                return next == PrimaryRuleGroupStatus::Revoked || next == PrimaryRuleGroupStatus::Expired;
            }
            PrimaryRuleGroupStatus::Revoked => {
                return next == PrimaryRuleGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRuleGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PrimaryRuleGroup {
        PrimaryRuleGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            region: self.region.clone(),
            category: self.category.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary rule group in status requested.
pub fn new_primary_rule_group(code: &str, name: &str) -> PrimaryRuleGroup {
    PrimaryRuleGroup {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        region: "".to_string(),
        category: "".to_string(),
        channel: "".to_string(),
        status: PrimaryRuleGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_rule_group("code-691", "name-226");
        assert_eq!(item.status, PrimaryRuleGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_rule_group("code-691", "name-226");
        if item.move_to(PrimaryRuleGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRuleGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_rule_group("code-691", "name-226");
        item.status = PrimaryRuleGroupStatus::Expired;
        assert!(
            item.move_to(PrimaryRuleGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
