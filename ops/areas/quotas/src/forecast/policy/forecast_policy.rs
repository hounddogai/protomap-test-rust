use super::*;

use acme_kit::text;

/// Where a forecast policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastPolicyStatus {
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

impl std::fmt::Display for ForecastPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPolicy {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub minimum: f64,
    pub owner_id: String,
    pub budget: f64,
    pub window_days: i64,
    pub visible: bool,
    pub updated_at: i64,
    pub status: ForecastPolicyStatus,
}

impl ForecastPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastPolicyStatus) -> bool {
        match self.status {
            ForecastPolicyStatus::Open => {
                match next {
                    ForecastPolicyStatus::Resolved | ForecastPolicyStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastPolicyStatus::InReview => {
                return next == ForecastPolicyStatus::Archived;
            }
            ForecastPolicyStatus::Resolved => {
                match next {
                    ForecastPolicyStatus::Reopened | ForecastPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastPolicyStatus::Reopened => {
                return next == ForecastPolicyStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast policy {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.maximum - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastPolicy {
        ForecastPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            window_days: self.window_days,
            visible: self.visible,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a forecast policy in status open.
pub fn new_forecast_policy(code: &str, name: &str) -> ForecastPolicy {
    ForecastPolicy {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        minimum: 0.0,
        owner_id: "".to_string(),
        budget: 0.0,
        window_days: 0,
        visible: false,
        updated_at: 0,
        status: ForecastPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_forecast_policy("code-714", "name-886");
        assert_eq!(item.status, ForecastPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_forecast_policy("code-714", "name-886");
        if item.move_to(ForecastPolicyStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastPolicyStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_forecast_policy("code-714", "name-886");
        item.status = ForecastPolicyStatus::Archived;
        assert!(
            item.move_to(ForecastPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
