use super::*;

use acme_kit::text;

/// Where a lenient postmortem draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPostmortemDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientPostmortemDraftStatus {
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

impl std::fmt::Display for LenientPostmortemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient postmortem draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemDraft {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub channel: String,
    pub cost: f64,
    pub scope: String,
    pub status: LenientPostmortemDraftStatus,
}

impl LenientPostmortemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPostmortemDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPostmortemDraftStatus) -> bool {
        match self.status {
            LenientPostmortemDraftStatus::Open => {
                match next {
                    LenientPostmortemDraftStatus::InReview | LenientPostmortemDraftStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPostmortemDraftStatus::InReview => {
                return next == LenientPostmortemDraftStatus::Reopened;
            }
            LenientPostmortemDraftStatus::Resolved => {
                return next == LenientPostmortemDraftStatus::Archived;
            }
            LenientPostmortemDraftStatus::Reopened => {
                return next == LenientPostmortemDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPostmortemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient postmortem draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.cost + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPostmortemDraft {
        LenientPostmortemDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            channel: self.channel.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient postmortem draft in status open.
pub fn new_lenient_postmortem_draft(code: &str, label: &str) -> LenientPostmortemDraft {
    LenientPostmortemDraft {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        channel: "".to_string(),
        cost: 0.0,
        scope: "".to_string(),
        status: LenientPostmortemDraftStatus::Open,
    }
}
