use super::*;

use acme_kit::text;

/// Where a strict reservation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReservationDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictReservationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictReservationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict reservation draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationDraft {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub budget: f64,
    pub burst: i64,
    pub duration_minutes: i64,
    pub score: f64,
    pub margin: f64,
    pub archived: bool,
    pub approved: bool,
    pub phone: String,
    pub status: StrictReservationDraftStatus,
}

impl StrictReservationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReservationDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReservationDraftStatus) -> bool {
        match self.status {
            StrictReservationDraftStatus::Draft => {
                match next {
                    StrictReservationDraftStatus::Active | StrictReservationDraftStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReservationDraftStatus::Pending => {
                match next {
                    StrictReservationDraftStatus::Suspended | StrictReservationDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReservationDraftStatus::Active => {
                return next == StrictReservationDraftStatus::Closed;
            }
            StrictReservationDraftStatus::Suspended => {
                match next {
                    StrictReservationDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictReservationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict reservation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict reservation draft", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.margin + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReservationDraft {
        StrictReservationDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            budget: self.budget,
            burst: self.burst,
            duration_minutes: self.duration_minutes,
            score: self.score,
            margin: self.margin,
            archived: self.archived,
            approved: self.approved,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict reservation draft in status draft.
pub fn new_strict_reservation_draft(code: &str, label: &str) -> StrictReservationDraft {
    StrictReservationDraft {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        budget: 0.0,
        burst: 0,
        duration_minutes: 0,
        score: 0.0,
        margin: 0.0,
        archived: false,
        approved: false,
        phone: "".to_string(),
        status: StrictReservationDraftStatus::Draft,
    }
}
