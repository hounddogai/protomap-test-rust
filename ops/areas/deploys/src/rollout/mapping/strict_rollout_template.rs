use super::*;

use acme_kit::text;

/// Where a strict rollout template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRolloutTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictRolloutTemplateStatus {
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

impl std::fmt::Display for StrictRolloutTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rollout template as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutTemplate {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub threshold: f64,
    pub priority: i64,
    pub ratio: f64,
    pub expires_at: i64,
    pub status: StrictRolloutTemplateStatus,
}

impl StrictRolloutTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRolloutTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRolloutTemplateStatus) -> bool {
        match self.status {
            StrictRolloutTemplateStatus::Planned => {
                match next {
                    StrictRolloutTemplateStatus::Scheduled | StrictRolloutTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutTemplateStatus::Scheduled => {
                match next {
                    StrictRolloutTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutTemplateStatus::InProgress => {
                match next {
                    StrictRolloutTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutTemplateStatus::Completed => {
                return next == StrictRolloutTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRolloutTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rollout template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.threshold * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRolloutTemplate {
        StrictRolloutTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            threshold: self.threshold,
            priority: self.priority,
            ratio: self.ratio,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict rollout template in status planned.
pub fn new_strict_rollout_template(code: &str, name: &str) -> StrictRolloutTemplate {
    StrictRolloutTemplate {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        threshold: 0.0,
        priority: 0,
        ratio: 0.0,
        expires_at: 0,
        status: StrictRolloutTemplateStatus::Planned,
    }
}
