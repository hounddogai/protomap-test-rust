use super::*;

use acme_kit::text;

/// Where a adaptive failover exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFailoverExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveFailoverExceptionStatus {
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

impl std::fmt::Display for AdaptiveFailoverExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive failover exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverException {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub namespace: String,
    pub sequence: i64,
    pub discount: f64,
    pub title: String,
    pub status: AdaptiveFailoverExceptionStatus,
}

impl AdaptiveFailoverException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFailoverExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFailoverExceptionStatus) -> bool {
        match self.status {
            AdaptiveFailoverExceptionStatus::Planned => {
                return next == AdaptiveFailoverExceptionStatus::Scheduled;
            }
            AdaptiveFailoverExceptionStatus::Scheduled => {
                return next == AdaptiveFailoverExceptionStatus::InProgress;
            }
            AdaptiveFailoverExceptionStatus::InProgress => {
                return next == AdaptiveFailoverExceptionStatus::Completed
                    || next == AdaptiveFailoverExceptionStatus::Cancelled;
            }
            AdaptiveFailoverExceptionStatus::Completed => {
                match next {
                    AdaptiveFailoverExceptionStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AdaptiveFailoverExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive failover exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive failover exception", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.discount * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFailoverException {
        AdaptiveFailoverException {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            sequence: self.sequence,
            discount: self.discount,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive failover exception in status planned.
pub fn new_adaptive_failover_exception(code: &str, label: &str) -> AdaptiveFailoverException {
    AdaptiveFailoverException {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        namespace: "".to_string(),
        sequence: 0,
        discount: 0.0,
        title: "".to_string(),
        status: AdaptiveFailoverExceptionStatus::Planned,
    }
}
