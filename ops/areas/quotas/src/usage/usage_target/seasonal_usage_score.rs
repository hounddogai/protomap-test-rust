use super::*;

use acme_kit::text;

/// Where a seasonal usage score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalUsageScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalUsageScoreStatus {
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

impl std::fmt::Display for SeasonalUsageScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal usage score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageScore {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub approved: bool,
    pub window_days: i64,
    pub description: String,
    pub status: SeasonalUsageScoreStatus,
}

impl SeasonalUsageScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalUsageScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalUsageScoreStatus) -> bool {
        match self.status {
            SeasonalUsageScoreStatus::Open => {
                match next {
                    SeasonalUsageScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageScoreStatus::InReview => {
                return next == SeasonalUsageScoreStatus::Resolved;
            }
            SeasonalUsageScoreStatus::Resolved => {
                return next == SeasonalUsageScoreStatus::Reopened || next == SeasonalUsageScoreStatus::Archived;
            }
            SeasonalUsageScoreStatus::Reopened => {
                return next == SeasonalUsageScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalUsageScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal usage score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.burst as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalUsageScore {
        SeasonalUsageScore {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            approved: self.approved,
            window_days: self.window_days,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal usage score in status open.
pub fn new_seasonal_usage_score(code: &str, name: &str) -> SeasonalUsageScore {
    SeasonalUsageScore {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        approved: false,
        window_days: 0,
        description: "".to_string(),
        status: SeasonalUsageScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_usage_score("code-801", "name-1");
        assert_eq!(item.status, SeasonalUsageScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_seasonal_usage_score("code-801", "name-1");
        if item.move_to(SeasonalUsageScoreStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalUsageScoreStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_usage_score("code-801", "name-1");
        item.status = SeasonalUsageScoreStatus::Archived;
        assert!(
            item.move_to(SeasonalUsageScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
