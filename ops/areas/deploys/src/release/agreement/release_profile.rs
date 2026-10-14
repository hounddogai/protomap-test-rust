use super::*;

use acme_kit::text;

/// Where a release profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ReleaseProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfile {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub attempts: i64,
    pub threshold: f64,
    pub window_days: i64,
    pub unit: String,
    pub ratio: f64,
    pub priority: i64,
    pub margin: f64,
    pub archived: bool,
    pub status: ReleaseProfileStatus,
}

impl ReleaseProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseProfileStatus) -> bool {
        match self.status {
            ReleaseProfileStatus::Draft => {
                match next {
                    ReleaseProfileStatus::Pending | ReleaseProfileStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseProfileStatus::Pending => {
                match next {
                    ReleaseProfileStatus::Active | ReleaseProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseProfileStatus::Active => {
                match next {
                    ReleaseProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseProfileStatus::Suspended => {
                return next == ReleaseProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "release profile", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.priority as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseProfile {
        ReleaseProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            attempts: self.attempts,
            threshold: self.threshold,
            window_days: self.window_days,
            unit: self.unit.clone(),
            ratio: self.ratio,
            priority: self.priority,
            margin: self.margin,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a release profile in status draft.
pub fn new_release_profile(code: &str, label: &str) -> ReleaseProfile {
    ReleaseProfile {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        attempts: 0,
        threshold: 0.0,
        window_days: 0,
        unit: "".to_string(),
        ratio: 0.0,
        priority: 0,
        margin: 0.0,
        archived: false,
        status: ReleaseProfileStatus::Draft,
    }
}
