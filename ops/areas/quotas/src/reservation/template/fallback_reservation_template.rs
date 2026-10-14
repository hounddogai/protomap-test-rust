use super::*;

use acme_kit::text;

/// Where a fallback reservation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReservationTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackReservationTemplateStatus {
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

impl std::fmt::Display for FallbackReservationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback reservation template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationTemplate {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub floor_value: f64,
    pub revision: i64,
    pub cost: f64,
    pub status: FallbackReservationTemplateStatus,
}

impl FallbackReservationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReservationTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReservationTemplateStatus) -> bool {
        match self.status {
            FallbackReservationTemplateStatus::Open => {
                match next {
                    FallbackReservationTemplateStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationTemplateStatus::InReview => {
                return next == FallbackReservationTemplateStatus::Resolved
                    || next == FallbackReservationTemplateStatus::Reopened;
            }
            FallbackReservationTemplateStatus::Resolved => {
                match next {
                    FallbackReservationTemplateStatus::Reopened | FallbackReservationTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationTemplateStatus::Reopened => {
                return next == FallbackReservationTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackReservationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback reservation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReservationTemplate {
        FallbackReservationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            floor_value: self.floor_value,
            revision: self.revision,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback reservation template in status open.
pub fn new_fallback_reservation_template(code: &str, title: &str) -> FallbackReservationTemplate {
    FallbackReservationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        floor_value: 0.0,
        revision: 0,
        cost: 0.0,
        status: FallbackReservationTemplateStatus::Open,
    }
}
