use super::*;

use acme_kit::text;

/// Where a overage cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl OverageCycleStatus {
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

impl std::fmt::Display for OverageCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageCycle {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub category: String,
    pub channel: String,
    pub pinned: bool,
    pub grace_days: i64,
    pub cluster: String,
    pub weight: f64,
    pub visible: bool,
    pub cost: f64,
    pub status: OverageCycleStatus,
}

impl OverageCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageCycleStatus) -> bool {
        match self.status {
            OverageCycleStatus::Idle => {
                return next == OverageCycleStatus::Stopped;
            }
            OverageCycleStatus::Warming => {
                match next {
                    OverageCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageCycleStatus::Ready => {
                match next {
                    OverageCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageCycleStatus::Draining => {
                match next {
                    OverageCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: OverageCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageCycle {
        OverageCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            category: self.category.clone(),
            channel: self.channel.clone(),
            pinned: self.pinned,
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            weight: self.weight,
            visible: self.visible,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a overage cycle in status idle.
pub fn new_overage_cycle(code: &str, label: &str) -> OverageCycle {
    OverageCycle {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        category: "".to_string(),
        channel: "".to_string(),
        pinned: false,
        grace_days: 0,
        cluster: "".to_string(),
        weight: 0.0,
        visible: false,
        cost: 0.0,
        status: OverageCycleStatus::Idle,
    }
}
