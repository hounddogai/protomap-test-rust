use super::*;

use acme_kit::text;

/// Where a alert draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertDraftStatus {
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

impl std::fmt::Display for AlertDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraft {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub reviewer: String,
    pub version: i64,
    pub channel: String,
    pub window_days: i64,
    pub archived: bool,
    pub cluster: String,
    pub limit: i64,
    pub burst: i64,
    pub status: AlertDraftStatus,
}

impl AlertDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertDraftStatus) -> bool {
        match self.status {
            AlertDraftStatus::Planned => {
                match next {
                    AlertDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertDraftStatus::Scheduled => {
                return next == AlertDraftStatus::Completed || next == AlertDraftStatus::Cancelled;
            }
            AlertDraftStatus::InProgress => {
                return next == AlertDraftStatus::Cancelled;
            }
            AlertDraftStatus::Completed => {
                return next == AlertDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertDraft {
        AlertDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            version: self.version,
            channel: self.channel.clone(),
            window_days: self.window_days,
            archived: self.archived,
            cluster: self.cluster.clone(),
            limit: self.limit,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a alert draft in status planned.
pub fn new_alert_draft(code: &str, label: &str) -> AlertDraft {
    AlertDraft {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        reviewer: "".to_string(),
        version: 0,
        channel: "".to_string(),
        window_days: 0,
        archived: false,
        cluster: "".to_string(),
        limit: 0,
        burst: 0,
        status: AlertDraftStatus::Planned,
    }
}
