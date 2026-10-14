use super::*;

use acme_kit::text;

/// Where a seasonal quote assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalQuoteAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalQuoteAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalQuoteAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal quote assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteAssignment {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub fingerprint: String,
    pub port: i64,
    pub status: SeasonalQuoteAssignmentStatus,
}

impl SeasonalQuoteAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalQuoteAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalQuoteAssignmentStatus) -> bool {
        match self.status {
            SeasonalQuoteAssignmentStatus::Proposed => {
                match next {
                    SeasonalQuoteAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQuoteAssignmentStatus::Approved => {
                match next {
                    SeasonalQuoteAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalQuoteAssignmentStatus::Rejected => {
                return next == SeasonalQuoteAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalQuoteAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal quote assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.port as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalQuoteAssignment {
        SeasonalQuoteAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal quote assignment in status proposed.
pub fn new_seasonal_quote_assignment(code: &str, title: &str) -> SeasonalQuoteAssignment {
    SeasonalQuoteAssignment {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        fingerprint: "".to_string(),
        port: 0,
        status: SeasonalQuoteAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_quote_assignment("code-734", "title-78");
        assert_eq!(item.status, SeasonalQuoteAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_quote_assignment("code-734", "title-78");
        if item.move_to(SeasonalQuoteAssignmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalQuoteAssignmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_quote_assignment("code-734", "title-78");
        item.status = SeasonalQuoteAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalQuoteAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
