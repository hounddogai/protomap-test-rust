use super::*;

use acme_kit::text;

/// Where a lenient peering draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPeeringDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientPeeringDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LenientPeeringDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient peering draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraft {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub priority: i64,
    pub rotation_days: i64,
    pub status: LenientPeeringDraftStatus,
}

impl LenientPeeringDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPeeringDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPeeringDraftStatus) -> bool {
        match self.status {
            LenientPeeringDraftStatus::New => {
                match next {
                    LenientPeeringDraftStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringDraftStatus::Queued => {
                match next {
                    LenientPeeringDraftStatus::Running | LenientPeeringDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringDraftStatus::Running => {
                return next == LenientPeeringDraftStatus::Failed;
            }
            LenientPeeringDraftStatus::Done => {
                return next == LenientPeeringDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPeeringDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient peering draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient peering draft", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPeeringDraft {
        LenientPeeringDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient peering draft in status new.
pub fn new_lenient_peering_draft(code: &str, title: &str) -> LenientPeeringDraft {
    LenientPeeringDraft {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        priority: 0,
        rotation_days: 0,
        status: LenientPeeringDraftStatus::New,
    }
}
