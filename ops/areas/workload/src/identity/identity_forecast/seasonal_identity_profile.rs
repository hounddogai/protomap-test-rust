use super::*;

use acme_kit::text;

/// Where a seasonal identity profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIdentityProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalIdentityProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalIdentityProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal identity profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfile {
    pub code: String,
    pub label: String,
    pub category: String,
    pub version: i64,
    pub ceiling: f64,
    pub channel: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub revision: i64,
    pub status: SeasonalIdentityProfileStatus,
}

impl SeasonalIdentityProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIdentityProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIdentityProfileStatus) -> bool {
        match self.status {
            SeasonalIdentityProfileStatus::Planned => {
                return next == SeasonalIdentityProfileStatus::InProgress
                    || next == SeasonalIdentityProfileStatus::Completed;
            }
            SeasonalIdentityProfileStatus::Scheduled => {
                match next {
                    SeasonalIdentityProfileStatus::InProgress | SeasonalIdentityProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityProfileStatus::InProgress => {
                return next == SeasonalIdentityProfileStatus::Cancelled;
            }
            SeasonalIdentityProfileStatus::Completed => {
                match next {
                    SeasonalIdentityProfileStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalIdentityProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal identity profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal identity profile", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIdentityProfile {
        SeasonalIdentityProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            version: self.version,
            ceiling: self.ceiling,
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal identity profile in status planned.
pub fn new_seasonal_identity_profile(code: &str, label: &str) -> SeasonalIdentityProfile {
    SeasonalIdentityProfile {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        version: 0,
        ceiling: 0.0,
        channel: "".to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        revision: 0,
        status: SeasonalIdentityProfileStatus::Planned,
    }
}
