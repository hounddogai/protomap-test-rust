use super::*;

use acme_kit::text;

/// Where a default release rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultReleaseRuleStatus {
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

impl std::fmt::Display for DefaultReleaseRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default release rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseRule {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub scope: String,
    pub reviewer: String,
    pub segment: String,
    pub status: DefaultReleaseRuleStatus,
}

impl DefaultReleaseRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseRuleStatus) -> bool {
        match self.status {
            DefaultReleaseRuleStatus::Planned => {
                return next == DefaultReleaseRuleStatus::Completed;
            }
            DefaultReleaseRuleStatus::Scheduled => {
                match next {
                    DefaultReleaseRuleStatus::InProgress | DefaultReleaseRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseRuleStatus::InProgress => {
                match next {
                    DefaultReleaseRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseRuleStatus::Completed => {
                return next == DefaultReleaseRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultReleaseRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release rule {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> DefaultReleaseRule {
        DefaultReleaseRule {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            scope: self.scope.clone(),
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default release rule in status planned.
pub fn new_default_release_rule(code: &str, name: &str) -> DefaultReleaseRule {
    DefaultReleaseRule {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        scope: "".to_string(),
        reviewer: "".to_string(),
        segment: "".to_string(),
        status: DefaultReleaseRuleStatus::Planned,
    }
}
