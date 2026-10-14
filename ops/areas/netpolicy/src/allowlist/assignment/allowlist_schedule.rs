use super::*;

use acme_kit::text;

/// Where a allowlist schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AllowlistScheduleStatus {
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

impl std::fmt::Display for AllowlistScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSchedule {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub channel: String,
    pub batch_size: i64,
    pub status: AllowlistScheduleStatus,
}

impl AllowlistSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistScheduleStatus) -> bool {
        match self.status {
            AllowlistScheduleStatus::Open => {
                return next == AllowlistScheduleStatus::InReview || next == AllowlistScheduleStatus::Resolved;
            }
            AllowlistScheduleStatus::InReview => {
                return next == AllowlistScheduleStatus::Reopened;
            }
            AllowlistScheduleStatus::Resolved => {
                return next == AllowlistScheduleStatus::Reopened || next == AllowlistScheduleStatus::Archived;
            }
            AllowlistScheduleStatus::Reopened => {
                match next {
                    AllowlistScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: AllowlistScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.updated_at as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistSchedule {
        AllowlistSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a allowlist schedule in status open.
pub fn new_allowlist_schedule(code: &str, label: &str) -> AllowlistSchedule {
    AllowlistSchedule {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        channel: "".to_string(),
        batch_size: 0,
        status: AllowlistScheduleStatus::Open,
    }
}
