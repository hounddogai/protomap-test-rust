use super::*;

use acme_kit::text;

/// Where a identity forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecast {
    pub code: String,
    pub label: String,
    pub description: String,
    pub cost: f64,
    pub name: String,
    pub status: IdentityForecastStatus,
}

impl IdentityForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityForecastStatus) -> bool {
        match self.status {
            IdentityForecastStatus::Requested => {
                match next {
                    IdentityForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityForecastStatus::Granted => {
                match next {
                    IdentityForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityForecastStatus::Revoked => {
                return next == IdentityForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> IdentityForecast {
        IdentityForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            cost: self.cost,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity forecast in status requested.
pub fn new_identity_forecast(code: &str, label: &str) -> IdentityForecast {
    IdentityForecast {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        cost: 0.0,
        name: "".to_string(),
        status: IdentityForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_forecast("code-505", "label-932");
        assert_eq!(item.status, IdentityForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_identity_forecast("code-505", "label-932");
        if item.move_to(IdentityForecastStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityForecastStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_forecast("code-505", "label-932");
        item.status = IdentityForecastStatus::Expired;
        assert!(
            item.move_to(IdentityForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
