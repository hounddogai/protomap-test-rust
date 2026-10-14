use super::*;

use acme_kit::text;

/// Where a adaptive forecast policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveForecastPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveForecastPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveForecastPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive forecast policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicy {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub rate: f64,
    pub country: String,
    pub approved: bool,
    pub quantity: i64,
    pub budget: f64,
    pub cluster: String,
    pub source: String,
    pub ceiling: f64,
    pub status: AdaptiveForecastPolicyStatus,
}

impl AdaptiveForecastPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveForecastPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveForecastPolicyStatus) -> bool {
        match self.status {
            AdaptiveForecastPolicyStatus::Created => {
                match next {
                    AdaptiveForecastPolicyStatus::Validated | AdaptiveForecastPolicyStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveForecastPolicyStatus::Validated => {
                return next == AdaptiveForecastPolicyStatus::Published
                    || next == AdaptiveForecastPolicyStatus::Retired;
            }
            AdaptiveForecastPolicyStatus::Published => {
                return next == AdaptiveForecastPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveForecastPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive forecast policy {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rate - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveForecastPolicy {
        AdaptiveForecastPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            rate: self.rate,
            country: self.country.clone(),
            approved: self.approved,
            quantity: self.quantity,
            budget: self.budget,
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive forecast policy in status created.
pub fn new_adaptive_forecast_policy(code: &str, title: &str) -> AdaptiveForecastPolicy {
    AdaptiveForecastPolicy {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        rate: 0.0,
        country: "".to_string(),
        approved: false,
        quantity: 0,
        budget: 0.0,
        cluster: "".to_string(),
        source: "".to_string(),
        ceiling: 0.0,
        status: AdaptiveForecastPolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_forecast_policy("code-21", "title-221");
        assert_eq!(item.status, AdaptiveForecastPolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_adaptive_forecast_policy("code-21", "title-221");
        if item.move_to(AdaptiveForecastPolicyStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveForecastPolicyStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_forecast_policy("code-21", "title-221");
        item.status = AdaptiveForecastPolicyStatus::Retired;
        assert!(
            item.move_to(AdaptiveForecastPolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
