use super::*;

use acme_kit::text;

/// Where a delegation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DelegationForecastStatus {
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

impl std::fmt::Display for DelegationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationForecast {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub cipher: String,
    pub title: String,
    pub sequence: i64,
    pub timezone: String,
    pub burst: i64,
    pub category: String,
    pub status: DelegationForecastStatus,
}

impl DelegationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationForecastStatus) -> bool {
        match self.status {
            DelegationForecastStatus::Open => {
                return next == DelegationForecastStatus::InReview || next == DelegationForecastStatus::Archived;
            }
            DelegationForecastStatus::InReview => {
                return next == DelegationForecastStatus::Resolved || next == DelegationForecastStatus::Reopened;
            }
            DelegationForecastStatus::Resolved => {
                match next {
                    DelegationForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationForecastStatus::Reopened => {
                return next == DelegationForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.amount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationForecast {
        DelegationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            cipher: self.cipher.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            timezone: self.timezone.clone(),
            burst: self.burst,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a delegation forecast in status open.
pub fn new_delegation_forecast(code: &str, label: &str) -> DelegationForecast {
    DelegationForecast {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        cipher: "".to_string(),
        title: "".to_string(),
        sequence: 0,
        timezone: "".to_string(),
        burst: 0,
        category: "".to_string(),
        status: DelegationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_delegation_forecast("code-993", "label-865");
        assert_eq!(item.status, DelegationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_delegation_forecast("code-993", "label-865");
        if item.move_to(DelegationForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_delegation_forecast("code-993", "label-865");
        item.status = DelegationForecastStatus::Archived;
        assert!(
            item.move_to(DelegationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
