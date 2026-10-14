use super::*;

use acme_kit::text;

/// Where a adaptive overage draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveOverageDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveOverageDraftStatus {
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

impl std::fmt::Display for AdaptiveOverageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive overage draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraft {
    pub code: String,
    pub title: String,
    pub label: String,
    pub amount: f64,
    pub batch_size: i64,
    pub contact_email: String,
    pub pinned: bool,
    pub status: AdaptiveOverageDraftStatus,
}

impl AdaptiveOverageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveOverageDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveOverageDraftStatus) -> bool {
        match self.status {
            AdaptiveOverageDraftStatus::Planned => {
                match next {
                    AdaptiveOverageDraftStatus::Scheduled | AdaptiveOverageDraftStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveOverageDraftStatus::Scheduled => {
                match next {
                    AdaptiveOverageDraftStatus::InProgress | AdaptiveOverageDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveOverageDraftStatus::InProgress => {
                match next {
                    AdaptiveOverageDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveOverageDraftStatus::Completed => {
                return next == AdaptiveOverageDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveOverageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive overage draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive overage draft", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.amount * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveOverageDraft {
        AdaptiveOverageDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            amount: self.amount,
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive overage draft in status planned.
pub fn new_adaptive_overage_draft(code: &str, title: &str) -> AdaptiveOverageDraft {
    AdaptiveOverageDraft {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        amount: 0.0,
        batch_size: 0,
        contact_email: "".to_string(),
        pinned: false,
        status: AdaptiveOverageDraftStatus::Planned,
    }
}
