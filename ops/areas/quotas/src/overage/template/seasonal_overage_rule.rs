use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalOverageRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalOverageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalOverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal overage rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageRule {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub notes: String,
    pub country: String,
    pub attempts: i64,
    pub updated_at: i64,
    pub status: SeasonalOverageRuleStatus,
}

impl SeasonalOverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalOverageRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalOverageRuleStatus) -> bool {
        match self.status {
            SeasonalOverageRuleStatus::Requested => {
                match next {
                    SeasonalOverageRuleStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalOverageRuleStatus::Granted => {
                return next == SeasonalOverageRuleStatus::Revoked || next == SeasonalOverageRuleStatus::Expired;
            }
            SeasonalOverageRuleStatus::Revoked => {
                return next == SeasonalOverageRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalOverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal overage rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalOverageRule {
        SeasonalOverageRule {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            notes: self.notes.clone(),
            country: self.country.clone(),
            attempts: self.attempts,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal overage rule in status requested.
pub fn new_seasonal_overage_rule(code: &str, name: &str) -> SeasonalOverageRule {
    SeasonalOverageRule {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        notes: "".to_string(),
        country: "".to_string(),
        attempts: 0,
        updated_at: 0,
        status: SeasonalOverageRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_overage_rule("code-458", "name-582");
        assert_eq!(item.status, SeasonalOverageRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_overage_rule("code-458", "name-582");
        if item.move_to(SeasonalOverageRuleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalOverageRuleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_overage_rule("code-458", "name-582");
        item.status = SeasonalOverageRuleStatus::Expired;
        assert!(
            item.move_to(SeasonalOverageRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
