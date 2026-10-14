use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled rollout rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRolloutRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledRolloutRollupStatus {
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

impl std::fmt::Display for ScheduledRolloutRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rollout rollup as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollup {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub priority: i64,
    pub attempts: i64,
    pub key_id: String,
    pub approved: bool,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub status: ScheduledRolloutRollupStatus,
}

impl ScheduledRolloutRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRolloutRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRolloutRollupStatus) -> bool {
        match self.status {
            ScheduledRolloutRollupStatus::Open => {
                return next == ScheduledRolloutRollupStatus::InReview;
            }
            ScheduledRolloutRollupStatus::InReview => {
                return next == ScheduledRolloutRollupStatus::Reopened;
            }
            ScheduledRolloutRollupStatus::Resolved => {
                return next == ScheduledRolloutRollupStatus::Reopened
                    || next == ScheduledRolloutRollupStatus::Archived;
            }
            ScheduledRolloutRollupStatus::Reopened => {
                match next {
                    ScheduledRolloutRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledRolloutRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollout rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled rollout rollup", self.code)
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

    pub fn effective_value(&self) -> f64 {
        ((self.attempts as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRolloutRollup {
        ScheduledRolloutRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            priority: self.priority,
            attempts: self.attempts,
            key_id: self.key_id.clone(),
            approved: self.approved,
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled rollout rollup in status open.
pub fn new_scheduled_rollout_rollup(code: &str, name: &str) -> ScheduledRolloutRollup {
    ScheduledRolloutRollup {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        priority: 0,
        attempts: 0,
        key_id: "".to_string(),
        approved: false,
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        status: ScheduledRolloutRollupStatus::Open,
    }
}
