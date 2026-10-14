use super::*;

use acme_kit::text;

/// Where a strict isolation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictIsolationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictIsolationForecastStatus {
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

impl std::fmt::Display for StrictIsolationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict isolation forecast as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecast {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub scope: String,
    pub score: f64,
    pub active: bool,
    pub contact_email: String,
    pub status: StrictIsolationForecastStatus,
}

impl StrictIsolationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictIsolationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictIsolationForecastStatus) -> bool {
        match self.status {
            StrictIsolationForecastStatus::Open => {
                return next == StrictIsolationForecastStatus::Resolved
                    || next == StrictIsolationForecastStatus::Archived;
            }
            StrictIsolationForecastStatus::InReview => {
                match next {
                    StrictIsolationForecastStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIsolationForecastStatus::Resolved => {
                match next {
                    StrictIsolationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIsolationForecastStatus::Reopened => {
                return next == StrictIsolationForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictIsolationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict isolation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> StrictIsolationForecast {
        StrictIsolationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            scope: self.scope.clone(),
            score: self.score,
            active: self.active,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict isolation forecast in status open.
pub fn new_strict_isolation_forecast(code: &str, label: &str) -> StrictIsolationForecast {
    StrictIsolationForecast {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        scope: "".to_string(),
        score: 0.0,
        active: false,
        contact_email: "".to_string(),
        status: StrictIsolationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_isolation_forecast("code-961", "label-738");
        assert_eq!(item.status, StrictIsolationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_isolation_forecast("code-961", "label-738");
        if item.move_to(StrictIsolationForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictIsolationForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_isolation_forecast("code-961", "label-738");
        item.status = StrictIsolationForecastStatus::Archived;
        assert!(
            item.move_to(StrictIsolationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
