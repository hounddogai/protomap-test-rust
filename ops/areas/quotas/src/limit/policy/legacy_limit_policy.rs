use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy limit policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyLimitPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyLimitPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyLimitPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy limit policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicy {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub grace_days: i64,
    pub minimum: f64,
    pub visible: bool,
    pub key_id: String,
    pub category: String,
    pub status: LegacyLimitPolicyStatus,
}

impl LegacyLimitPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyLimitPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyLimitPolicyStatus) -> bool {
        match self.status {
            LegacyLimitPolicyStatus::Proposed => {
                return next == LegacyLimitPolicyStatus::Rejected || next == LegacyLimitPolicyStatus::Withdrawn;
            }
            LegacyLimitPolicyStatus::Approved => {
                match next {
                    LegacyLimitPolicyStatus::Rejected | LegacyLimitPolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyLimitPolicyStatus::Rejected => {
                match next {
                    LegacyLimitPolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyLimitPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy limit policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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
        (self.discount * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyLimitPolicy {
        LegacyLimitPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            discount: self.discount,
            grace_days: self.grace_days,
            minimum: self.minimum,
            visible: self.visible,
            key_id: self.key_id.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy limit policy in status proposed.
pub fn new_legacy_limit_policy(code: &str, label: &str) -> LegacyLimitPolicy {
    LegacyLimitPolicy {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        discount: 0.0,
        grace_days: 0,
        minimum: 0.0,
        visible: false,
        key_id: "".to_string(),
        category: "".to_string(),
        status: LegacyLimitPolicyStatus::Proposed,
    }
}
