use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RuleGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule group as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleGroup {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub discount: f64,
    pub namespace: String,
    pub floor_value: f64,
    pub window_days: i64,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: RuleGroupStatus,
}

impl RuleGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleGroupStatus) -> bool {
        match self.status {
            RuleGroupStatus::Open => {
                match next {
                    RuleGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleGroupStatus::InReview => {
                match next {
                    RuleGroupStatus::Resolved | RuleGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleGroupStatus::Resolved => {
                match next {
                    RuleGroupStatus::Reopened | RuleGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleGroupStatus::Reopened => {
                match next {
                    RuleGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: RuleGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule group", self.code)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rate + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleGroup {
        RuleGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            discount: self.discount,
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            window_days: self.window_days,
            rate: self.rate,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rule group in status open.
pub fn new_rule_group(code: &str, name: &str) -> RuleGroup {
    RuleGroup {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        discount: 0.0,
        namespace: "".to_string(),
        floor_value: 0.0,
        window_days: 0,
        rate: 0.0,
        attributes: BTreeMap::new(),
        status: RuleGroupStatus::Open,
    }
}
