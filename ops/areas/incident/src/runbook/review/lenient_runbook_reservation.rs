use super::*;

use acme_kit::text;

/// Where a lenient runbook reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRunbookReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientRunbookReservationStatus {
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

impl std::fmt::Display for LenientRunbookReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient runbook reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRunbookReservation {
    pub code: String,
    pub title: String,
    pub description: String,
    pub score: f64,
    pub margin: f64,
    pub owner_id: String,
    pub status: LenientRunbookReservationStatus,
}

impl LenientRunbookReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRunbookReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRunbookReservationStatus) -> bool {
        match self.status {
            LenientRunbookReservationStatus::New => {
                match next {
                    LenientRunbookReservationStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRunbookReservationStatus::Queued => {
                return next == LenientRunbookReservationStatus::Done
                    || next == LenientRunbookReservationStatus::Failed;
            }
            LenientRunbookReservationStatus::Running => {
                return next == LenientRunbookReservationStatus::Done;
            }
            LenientRunbookReservationStatus::Done => {
                match next {
                    LenientRunbookReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: LenientRunbookReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient runbook reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient runbook reservation", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.margin + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRunbookReservation {
        LenientRunbookReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            score: self.score,
            margin: self.margin,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient runbook reservation in status new.
pub fn new_lenient_runbook_reservation(code: &str, title: &str) -> LenientRunbookReservation {
    LenientRunbookReservation {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        score: 0.0,
        margin: 0.0,
        owner_id: "".to_string(),
        status: LenientRunbookReservationStatus::New,
    }
}
