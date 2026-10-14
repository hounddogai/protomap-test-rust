use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RuleBundleStatus {
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

impl std::fmt::Display for RuleBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule bundle as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundle {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub score: f64,
    pub attributes: BTreeMap<String, String>,
    pub margin: f64,
    pub status: RuleBundleStatus,
}

impl RuleBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleBundleStatus) -> bool {
        match self.status {
            RuleBundleStatus::Draft => {
                return next == RuleBundleStatus::Active;
            }
            RuleBundleStatus::Pending => {
                return next == RuleBundleStatus::Active || next == RuleBundleStatus::Closed;
            }
            RuleBundleStatus::Active => {
                return next == RuleBundleStatus::Suspended;
            }
            RuleBundleStatus::Suspended => {
                match next {
                    RuleBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: RuleBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule bundle", self.code)
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

    pub fn net_load(&self) -> f64 {
        ((self.margin + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleBundle {
        RuleBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            score: self.score,
            attributes: self.attributes.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rule bundle in status draft.
pub fn new_rule_bundle(code: &str, name: &str) -> RuleBundle {
    RuleBundle {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        score: 0.0,
        attributes: BTreeMap::new(),
        margin: 0.0,
        status: RuleBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rule_bundle("code-312", "name-665");
        assert_eq!(item.status, RuleBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rule_bundle("code-312", "name-665");
        if item.move_to(RuleBundleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleBundleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rule_bundle("code-312", "name-665");
        item.status = RuleBundleStatus::Closed;
        assert!(item.move_to(RuleBundleStatus::Draft).is_err(), "expected move to fail");
    }
}
