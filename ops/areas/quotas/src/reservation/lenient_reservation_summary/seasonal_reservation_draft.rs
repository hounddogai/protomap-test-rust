use super::*;

use acme_kit::text;

/// Where a seasonal reservation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReservationDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalReservationDraftStatus {
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

impl std::fmt::Display for SeasonalReservationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal reservation draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationDraft {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub cluster: String,
    pub scope: String,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub segment: String,
    pub burst: i64,
    pub status: SeasonalReservationDraftStatus,
}

impl SeasonalReservationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReservationDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReservationDraftStatus) -> bool {
        match self.status {
            SeasonalReservationDraftStatus::Draft => {
                return next == SeasonalReservationDraftStatus::Pending
                    || next == SeasonalReservationDraftStatus::Active;
            }
            SeasonalReservationDraftStatus::Pending => {
                return next == SeasonalReservationDraftStatus::Suspended;
            }
            SeasonalReservationDraftStatus::Active => {
                match next {
                    SeasonalReservationDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReservationDraftStatus::Suspended => {
                return next == SeasonalReservationDraftStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalReservationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal reservation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalReservationDraft {
        SeasonalReservationDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            tags: self.tags.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal reservation draft in status draft.
pub fn new_seasonal_reservation_draft(code: &str, title: &str) -> SeasonalReservationDraft {
    SeasonalReservationDraft {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        cluster: "".to_string(),
        scope: "".to_string(),
        tags: Vec::new(),
        quantity: 0,
        segment: "".to_string(),
        burst: 0,
        status: SeasonalReservationDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_reservation_draft("code-65", "title-317");
        assert_eq!(item.status, SeasonalReservationDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_seasonal_reservation_draft("code-65", "title-317");
        if item.move_to(SeasonalReservationDraftStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReservationDraftStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_reservation_draft("code-65", "title-317");
        item.status = SeasonalReservationDraftStatus::Closed;
        assert!(
            item.move_to(SeasonalReservationDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
