use super::*;

use acme_kit::text;

/// Where a rolling verifier forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingVerifierForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingVerifierForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingVerifierForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling verifier forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecast {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub grace_days: i64,
    pub notes: String,
    pub category: String,
    pub cluster: String,
    pub expires_at: i64,
    pub cost: f64,
    pub status: RollingVerifierForecastStatus,
}

impl RollingVerifierForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingVerifierForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingVerifierForecastStatus) -> bool {
        match self.status {
            RollingVerifierForecastStatus::Proposed => {
                match next {
                    RollingVerifierForecastStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierForecastStatus::Approved => {
                match next {
                    RollingVerifierForecastStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingVerifierForecastStatus::Rejected => {
                return next == RollingVerifierForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingVerifierForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling verifier forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.cost * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingVerifierForecast {
        RollingVerifierForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            grace_days: self.grace_days,
            notes: self.notes.clone(),
            category: self.category.clone(),
            cluster: self.cluster.clone(),
            expires_at: self.expires_at,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling verifier forecast in status proposed.
pub fn new_rolling_verifier_forecast(code: &str, label: &str) -> RollingVerifierForecast {
    RollingVerifierForecast {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        grace_days: 0,
        notes: "".to_string(),
        category: "".to_string(),
        cluster: "".to_string(),
        expires_at: 0,
        cost: 0.0,
        status: RollingVerifierForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_verifier_forecast("code-661", "label-936");
        assert_eq!(item.status, RollingVerifierForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_verifier_forecast("code-661", "label-936");
        if item.move_to(RollingVerifierForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingVerifierForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_verifier_forecast("code-661", "label-936");
        item.status = RollingVerifierForecastStatus::Withdrawn;
        assert!(
            item.move_to(RollingVerifierForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
