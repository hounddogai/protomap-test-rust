use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule override as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleOverride {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub status: RuleOverrideStatus,
}

impl RuleOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleOverrideStatus) -> bool {
        match self.status {
            RuleOverrideStatus::Requested => {
                match next {
                    RuleOverrideStatus::Revoked | RuleOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleOverrideStatus::Granted => {
                return next == RuleOverrideStatus::Revoked;
            }
            RuleOverrideStatus::Revoked => {
                return next == RuleOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RuleOverride {
        RuleOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule override in status requested.
pub fn new_rule_override(code: &str, title: &str) -> RuleOverride {
    RuleOverride {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        score: 0.0,
        status: RuleOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_override("code-737", "title-593");
        assert_eq!(item.status, RuleOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rule_override("code-737", "title-593");
        if item.move_to(RuleOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_override("code-737", "title-593");
        item.status = RuleOverrideStatus::Expired;
        assert!(
            item.move_to(RuleOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
