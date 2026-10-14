use super::*;

use acme_kit::text;

/// Where a primary reservation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryReservationVarianceStatus {
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

impl std::fmt::Display for PrimaryReservationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary reservation variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationVariance {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub revision: i64,
    pub archived: bool,
    pub status: PrimaryReservationVarianceStatus,
}

impl PrimaryReservationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationVarianceStatus) -> bool {
        match self.status {
            PrimaryReservationVarianceStatus::Open => {
                match next {
                    PrimaryReservationVarianceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationVarianceStatus::InReview => {
                match next {
                    PrimaryReservationVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationVarianceStatus::Resolved => {
                match next {
                    PrimaryReservationVarianceStatus::Reopened | PrimaryReservationVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationVarianceStatus::Reopened => {
                return next == PrimaryReservationVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryReservationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation variance {} cannot move from {} to {}",
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.revision as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationVariance {
        PrimaryReservationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            revision: self.revision,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary reservation variance in status open.
pub fn new_primary_reservation_variance(code: &str, title: &str) -> PrimaryReservationVariance {
    PrimaryReservationVariance {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        revision: 0,
        archived: false,
        status: PrimaryReservationVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_reservation_variance("code-248", "title-379");
        assert_eq!(item.status, PrimaryReservationVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_primary_reservation_variance("code-248", "title-379");
        if item.move_to(PrimaryReservationVarianceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationVarianceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_reservation_variance("code-248", "title-379");
        item.status = PrimaryReservationVarianceStatus::Archived;
        assert!(
            item.move_to(PrimaryReservationVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
