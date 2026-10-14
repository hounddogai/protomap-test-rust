use super::*;

use acme_kit::text;

/// Where a scheduled egress profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEgressProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledEgressProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledEgressProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled egress profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfile {
    pub code: String,
    pub title: String,
    pub country: String,
    pub revision: i64,
    pub channel: String,
    pub quantity: i64,
    pub margin: f64,
    pub active: bool,
    pub weight: f64,
    pub status: ScheduledEgressProfileStatus,
}

impl ScheduledEgressProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEgressProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEgressProfileStatus) -> bool {
        match self.status {
            ScheduledEgressProfileStatus::New => {
                match next {
                    ScheduledEgressProfileStatus::Running | ScheduledEgressProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressProfileStatus::Queued => {
                match next {
                    ScheduledEgressProfileStatus::Running | ScheduledEgressProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressProfileStatus::Running => {
                match next {
                    ScheduledEgressProfileStatus::Done | ScheduledEgressProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEgressProfileStatus::Done => {
                return next == ScheduledEgressProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledEgressProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled egress profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.margin * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledEgressProfile {
        ScheduledEgressProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            revision: self.revision,
            channel: self.channel.clone(),
            quantity: self.quantity,
            margin: self.margin,
            active: self.active,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled egress profile in status new.
pub fn new_scheduled_egress_profile(code: &str, title: &str) -> ScheduledEgressProfile {
    ScheduledEgressProfile {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        revision: 0,
        channel: "".to_string(),
        quantity: 0,
        margin: 0.0,
        active: false,
        weight: 0.0,
        status: ScheduledEgressProfileStatus::New,
    }
}
