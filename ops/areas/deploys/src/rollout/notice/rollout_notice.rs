use super::*;

use acme_kit::text;

/// Where a rollout notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutNoticeStatus {
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

impl std::fmt::Display for RolloutNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNotice {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub floor_value: f64,
    pub margin: f64,
    pub source: String,
    pub scope: String,
    pub namespace: String,
    pub weight: f64,
    pub status: RolloutNoticeStatus,
}

impl RolloutNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutNoticeStatus) -> bool {
        match self.status {
            RolloutNoticeStatus::Open => {
                match next {
                    RolloutNoticeStatus::Resolved | RolloutNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutNoticeStatus::InReview => {
                return next == RolloutNoticeStatus::Reopened || next == RolloutNoticeStatus::Archived;
            }
            RolloutNoticeStatus::Resolved => {
                return next == RolloutNoticeStatus::Reopened;
            }
            RolloutNoticeStatus::Reopened => {
                match next {
                    RolloutNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: RolloutNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout notice", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.floor_value * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutNotice {
        RolloutNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            floor_value: self.floor_value,
            margin: self.margin,
            source: self.source.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout notice in status open.
pub fn new_rollout_notice(code: &str, title: &str) -> RolloutNotice {
    RolloutNotice {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        floor_value: 0.0,
        margin: 0.0,
        source: "".to_string(),
        scope: "".to_string(),
        namespace: "".to_string(),
        weight: 0.0,
        status: RolloutNoticeStatus::Open,
    }
}
