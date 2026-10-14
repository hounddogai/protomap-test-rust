use super::*;

use acme_kit::text;

/// Where a grant forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GrantForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GrantForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecast {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub budget: f64,
    pub quota_units: i64,
    pub limit: i64,
    pub external_ref: String,
    pub checksum: String,
    pub namespace: String,
    pub status: GrantForecastStatus,
}

impl GrantForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantForecastStatus) -> bool {
        match self.status {
            GrantForecastStatus::Created => {
                return next == GrantForecastStatus::Validated || next == GrantForecastStatus::Retired;
            }
            GrantForecastStatus::Validated => {
                return next == GrantForecastStatus::Retired;
            }
            GrantForecastStatus::Published => {
                match next {
                    GrantForecastStatus::Retired => {
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

    pub fn move_to(&mut self, next: GrantForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quota_units as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantForecast {
        GrantForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            budget: self.budget,
            quota_units: self.quota_units,
            limit: self.limit,
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a grant forecast in status created.
pub fn new_grant_forecast(code: &str, name: &str) -> GrantForecast {
    GrantForecast {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        budget: 0.0,
        quota_units: 0,
        limit: 0,
        external_ref: "".to_string(),
        checksum: "".to_string(),
        namespace: "".to_string(),
        status: GrantForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_grant_forecast("code-214", "name-614");
        assert_eq!(item.status, GrantForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_grant_forecast("code-214", "name-614");
        if item.move_to(GrantForecastStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantForecastStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_grant_forecast("code-214", "name-614");
        item.status = GrantForecastStatus::Retired;
        assert!(
            item.move_to(GrantForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
