use super::*;

use acme_kit::text;

/// Where a scheduled probe mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledProbeMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledProbeMappingStatus {
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

impl std::fmt::Display for ScheduledProbeMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled probe mapping as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMapping {
    pub code: String,
    pub title: String,
    pub host: String,
    pub archived: bool,
    pub region: String,
    pub quantity: i64,
    pub status: ScheduledProbeMappingStatus,
}

impl ScheduledProbeMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledProbeMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledProbeMappingStatus) -> bool {
        match self.status {
            ScheduledProbeMappingStatus::Idle => {
                return next == ScheduledProbeMappingStatus::Stopped;
            }
            ScheduledProbeMappingStatus::Warming => {
                match next {
                    ScheduledProbeMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeMappingStatus::Ready => {
                return next == ScheduledProbeMappingStatus::Draining;
            }
            ScheduledProbeMappingStatus::Draining => {
                return next == ScheduledProbeMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledProbeMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled probe mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled probe mapping", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ScheduledProbeMapping {
        ScheduledProbeMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            archived: self.archived,
            region: self.region.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled probe mapping in status idle.
pub fn new_scheduled_probe_mapping(code: &str, title: &str) -> ScheduledProbeMapping {
    ScheduledProbeMapping {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        archived: false,
        region: "".to_string(),
        quantity: 0,
        status: ScheduledProbeMappingStatus::Idle,
    }
}
