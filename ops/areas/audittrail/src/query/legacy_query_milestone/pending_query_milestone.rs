use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending query milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQueryMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingQueryMilestoneStatus {
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

impl std::fmt::Display for PendingQueryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending query milestone as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMilestone {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub ratio: f64,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub quota_units: i64,
    pub sequence: i64,
    pub status: PendingQueryMilestoneStatus,
}

impl PendingQueryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQueryMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQueryMilestoneStatus) -> bool {
        match self.status {
            PendingQueryMilestoneStatus::Idle => {
                return next == PendingQueryMilestoneStatus::Ready || next == PendingQueryMilestoneStatus::Stopped;
            }
            PendingQueryMilestoneStatus::Warming => {
                return next == PendingQueryMilestoneStatus::Ready || next == PendingQueryMilestoneStatus::Draining;
            }
            PendingQueryMilestoneStatus::Ready => {
                match next {
                    PendingQueryMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQueryMilestoneStatus::Draining => {
                match next {
                    PendingQueryMilestoneStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingQueryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending query milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending query milestone", self.code)
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

    pub fn projected_value(&self) -> f64 {
        ((self.burst as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQueryMilestone {
        PendingQueryMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            ratio: self.ratio,
            burst: self.burst,
            attributes: self.attributes.clone(),
            quota_units: self.quota_units,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending query milestone in status idle.
pub fn new_pending_query_milestone(code: &str, label: &str) -> PendingQueryMilestone {
    PendingQueryMilestone {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        ratio: 0.0,
        burst: 0,
        attributes: BTreeMap::new(),
        quota_units: 0,
        sequence: 0,
        status: PendingQueryMilestoneStatus::Idle,
    }
}
