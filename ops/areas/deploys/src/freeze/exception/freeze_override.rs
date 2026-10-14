use super::*;

use acme_kit::text;

/// Where a freeze override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeOverrideStatus {
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

impl std::fmt::Display for FreezeOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeOverride {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub summary: String,
    pub duration_minutes: i64,
    pub version: i64,
    pub status: FreezeOverrideStatus,
}

impl FreezeOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeOverrideStatus) -> bool {
        match self.status {
            FreezeOverrideStatus::Open => {
                match next {
                    FreezeOverrideStatus::Reopened | FreezeOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeOverrideStatus::InReview => {
                match next {
                    FreezeOverrideStatus::Resolved | FreezeOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeOverrideStatus::Resolved => {
                match next {
                    FreezeOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeOverrideStatus::Reopened => {
                return next == FreezeOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze override", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.version as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeOverride {
        FreezeOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a freeze override in status open.
pub fn new_freeze_override(code: &str, title: &str) -> FreezeOverride {
    FreezeOverride {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        summary: "".to_string(),
        duration_minutes: 0,
        version: 0,
        status: FreezeOverrideStatus::Open,
    }
}
