use super::*;

use acme_kit::text;

/// Where a primary overage forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryOverageForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryOverageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryOverageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary overage forecast as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageForecast {
    pub code: String,
    pub label: String,
    pub host: String,
    pub updated_at: i64,
    pub issuer: String,
    pub reviewer: String,
    pub currency: String,
    pub external_ref: String,
    pub status: PrimaryOverageForecastStatus,
}

impl PrimaryOverageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryOverageForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryOverageForecastStatus) -> bool {
        match self.status {
            PrimaryOverageForecastStatus::Requested => {
                match next {
                    PrimaryOverageForecastStatus::Granted | PrimaryOverageForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryOverageForecastStatus::Granted => {
                return next == PrimaryOverageForecastStatus::Revoked || next == PrimaryOverageForecastStatus::Expired;
            }
            PrimaryOverageForecastStatus::Revoked => {
                match next {
                    PrimaryOverageForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryOverageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary overage forecast {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> PrimaryOverageForecast {
        PrimaryOverageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary overage forecast in status requested.
pub fn new_primary_overage_forecast(code: &str, label: &str) -> PrimaryOverageForecast {
    PrimaryOverageForecast {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        updated_at: 0,
        issuer: "".to_string(),
        reviewer: "".to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        status: PrimaryOverageForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_overage_forecast("code-591", "label-981");
        assert_eq!(item.status, PrimaryOverageForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_overage_forecast("code-591", "label-981");
        if item.move_to(PrimaryOverageForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryOverageForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_overage_forecast("code-591", "label-981");
        item.status = PrimaryOverageForecastStatus::Expired;
        assert!(
            item.move_to(PrimaryOverageForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
