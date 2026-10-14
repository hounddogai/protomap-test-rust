use super::*;

use acme_kit::text;

/// Where a rollout template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RolloutTemplateStatus {
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

impl std::fmt::Display for RolloutTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTemplate {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub locale: String,
    pub window_days: i64,
    pub owner_id: String,
    pub status: RolloutTemplateStatus,
}

impl RolloutTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutTemplateStatus) -> bool {
        match self.status {
            RolloutTemplateStatus::Open => {
                match next {
                    RolloutTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutTemplateStatus::InReview => {
                return next == RolloutTemplateStatus::Reopened || next == RolloutTemplateStatus::Archived;
            }
            RolloutTemplateStatus::Resolved => {
                match next {
                    RolloutTemplateStatus::Reopened | RolloutTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutTemplateStatus::Reopened => {
                return next == RolloutTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.starts_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutTemplate {
        RolloutTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            window_days: self.window_days,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rollout template in status open.
pub fn new_rollout_template(code: &str, label: &str) -> RolloutTemplate {
    RolloutTemplate {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        locale: "".to_string(),
        window_days: 0,
        owner_id: "".to_string(),
        status: RolloutTemplateStatus::Open,
    }
}
