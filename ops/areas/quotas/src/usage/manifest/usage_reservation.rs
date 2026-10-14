use super::*;

use acme_kit::text;

/// Where a usage reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageReservationStatus {
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

impl std::fmt::Display for UsageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageReservation {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub weight: f64,
    pub scope: String,
    pub status: UsageReservationStatus,
}

impl UsageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageReservationStatus) -> bool {
        match self.status {
            UsageReservationStatus::Open => {
                match next {
                    UsageReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageReservationStatus::InReview => {
                match next {
                    UsageReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageReservationStatus::Resolved => {
                match next {
                    UsageReservationStatus::Reopened | UsageReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageReservationStatus::Reopened => {
                match next {
                    UsageReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: UsageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.weight * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageReservation {
        UsageReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            weight: self.weight,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage reservation in status open.
pub fn new_usage_reservation(code: &str, title: &str) -> UsageReservation {
    UsageReservation {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        weight: 0.0,
        scope: "".to_string(),
        status: UsageReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_usage_reservation("code-554", "title-643");
        assert_eq!(item.status, UsageReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_usage_reservation("code-554", "title-643");
        if item.move_to(UsageReservationStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageReservationStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_usage_reservation("code-554", "title-643");
        item.status = UsageReservationStatus::Archived;
        assert!(
            item.move_to(UsageReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
