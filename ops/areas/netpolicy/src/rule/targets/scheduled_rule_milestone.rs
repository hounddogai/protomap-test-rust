use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled rule milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRuleMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledRuleMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledRuleMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rule milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestone {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub starts_at: i64,
    pub locale: String,
    pub status: ScheduledRuleMilestoneStatus,
}

impl ScheduledRuleMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRuleMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRuleMilestoneStatus) -> bool {
        match self.status {
            ScheduledRuleMilestoneStatus::Created => {
                match next {
                    ScheduledRuleMilestoneStatus::Validated | ScheduledRuleMilestoneStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRuleMilestoneStatus::Validated => {
                match next {
                    ScheduledRuleMilestoneStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRuleMilestoneStatus::Published => {
                return next == ScheduledRuleMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRuleMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rule milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        self.starts_at as f64
    }

    pub fn copy(&self) -> ScheduledRuleMilestone {
        ScheduledRuleMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled rule milestone in status created.
pub fn new_scheduled_rule_milestone(code: &str, label: &str) -> ScheduledRuleMilestone {
    ScheduledRuleMilestone {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        starts_at: 0,
        locale: "".to_string(),
        status: ScheduledRuleMilestoneStatus::Created,
    }
}
