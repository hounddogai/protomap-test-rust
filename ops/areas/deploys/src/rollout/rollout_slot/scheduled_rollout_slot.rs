use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled rollout slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRolloutSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledRolloutSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledRolloutSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled rollout slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlot {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub timezone: String,
    pub attempts: i64,
    pub retries: i64,
    pub summary: String,
    pub cost: f64,
    pub status: ScheduledRolloutSlotStatus,
}

impl ScheduledRolloutSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRolloutSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRolloutSlotStatus) -> bool {
        match self.status {
            ScheduledRolloutSlotStatus::Idle => {
                match next {
                    ScheduledRolloutSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRolloutSlotStatus::Warming => {
                return next == ScheduledRolloutSlotStatus::Ready;
            }
            ScheduledRolloutSlotStatus::Ready => {
                match next {
                    ScheduledRolloutSlotStatus::Draining | ScheduledRolloutSlotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRolloutSlotStatus::Draining => {
                return next == ScheduledRolloutSlotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRolloutSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollout slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRolloutSlot {
        ScheduledRolloutSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            retries: self.retries,
            summary: self.summary.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled rollout slot in status idle.
pub fn new_scheduled_rollout_slot(code: &str, label: &str) -> ScheduledRolloutSlot {
    ScheduledRolloutSlot {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        host: "".to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        timezone: "".to_string(),
        attempts: 0,
        retries: 0,
        summary: "".to_string(),
        cost: 0.0,
        status: ScheduledRolloutSlotStatus::Idle,
    }
}
