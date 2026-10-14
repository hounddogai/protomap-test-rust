use super::*;

use acme_kit::text;

/// Where a regional forecast request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalForecastRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalForecastRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalForecastRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional forecast request as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastRequest {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub quantity: i64,
    pub key_id: String,
    pub grace_days: i64,
    pub name: String,
    pub external_ref: String,
    pub timezone: String,
    pub status: RegionalForecastRequestStatus,
}

impl RegionalForecastRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalForecastRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalForecastRequestStatus) -> bool {
        match self.status {
            RegionalForecastRequestStatus::Requested => {
                return next == RegionalForecastRequestStatus::Granted
                    || next == RegionalForecastRequestStatus::Revoked;
            }
            RegionalForecastRequestStatus::Granted => {
                match next {
                    RegionalForecastRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalForecastRequestStatus::Revoked => {
                match next {
                    RegionalForecastRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalForecastRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional forecast request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional forecast request", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalForecastRequest {
        RegionalForecastRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional forecast request in status requested.
pub fn new_regional_forecast_request(code: &str, label: &str) -> RegionalForecastRequest {
    RegionalForecastRequest {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        quantity: 0,
        key_id: "".to_string(),
        grace_days: 0,
        name: "".to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        status: RegionalForecastRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_forecast_request("code-789", "label-460");
        assert_eq!(item.status, RegionalForecastRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_forecast_request("code-789", "label-460");
        if item.move_to(RegionalForecastRequestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalForecastRequestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_forecast_request("code-789", "label-460");
        item.status = RegionalForecastRequestStatus::Expired;
        assert!(
            item.move_to(RegionalForecastRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
