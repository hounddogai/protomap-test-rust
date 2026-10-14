use super::*;

use acme_kit::text;

/// Where a seasonal tenant forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTenantForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalTenantForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalTenantForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal tenant forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecast {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub currency: String,
    pub port: i64,
    pub reviewer: String,
    pub namespace: String,
    pub revision: i64,
    pub ceiling: f64,
    pub notes: String,
    pub status: SeasonalTenantForecastStatus,
}

impl SeasonalTenantForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTenantForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTenantForecastStatus) -> bool {
        match self.status {
            SeasonalTenantForecastStatus::Requested => {
                return next == SeasonalTenantForecastStatus::Granted || next == SeasonalTenantForecastStatus::Revoked;
            }
            SeasonalTenantForecastStatus::Granted => {
                match next {
                    SeasonalTenantForecastStatus::Revoked | SeasonalTenantForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTenantForecastStatus::Revoked => {
                match next {
                    SeasonalTenantForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalTenantForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal tenant forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal tenant forecast", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.revision as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTenantForecast {
        SeasonalTenantForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            port: self.port,
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            revision: self.revision,
            ceiling: self.ceiling,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a seasonal tenant forecast in status requested.
pub fn new_seasonal_tenant_forecast(code: &str, name: &str) -> SeasonalTenantForecast {
    SeasonalTenantForecast {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        currency: "".to_string(),
        port: 0,
        reviewer: "".to_string(),
        namespace: "".to_string(),
        revision: 0,
        ceiling: 0.0,
        notes: "".to_string(),
        status: SeasonalTenantForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_tenant_forecast("code-964", "name-53");
        assert_eq!(item.status, SeasonalTenantForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_tenant_forecast("code-964", "name-53");
        if item.move_to(SeasonalTenantForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTenantForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_tenant_forecast("code-964", "name-53");
        item.status = SeasonalTenantForecastStatus::Expired;
        assert!(
            item.move_to(SeasonalTenantForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
