use super::*;

use acme_kit::text;

/// Where a usage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRule {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub quota_units: i64,
    pub currency: String,
    pub ceiling: f64,
    pub region: String,
    pub checksum: String,
    pub status: UsageRuleStatus,
}

impl UsageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageRuleStatus) -> bool {
        match self.status {
            UsageRuleStatus::Requested => {
                match next {
                    UsageRuleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageRuleStatus::Granted => {
                match next {
                    UsageRuleStatus::Revoked | UsageRuleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageRuleStatus::Revoked => {
                return next == UsageRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ceiling - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageRule {
        UsageRule {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a usage rule in status requested.
pub fn new_usage_rule(code: &str, title: &str) -> UsageRule {
    UsageRule {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        quota_units: 0,
        currency: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        checksum: "".to_string(),
        status: UsageRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_rule("code-941", "title-10");
        assert_eq!(item.status, UsageRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_usage_rule("code-941", "title-10");
        if item.move_to(UsageRuleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageRuleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_rule("code-941", "title-10");
        item.status = UsageRuleStatus::Expired;
        assert!(
            item.move_to(UsageRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
