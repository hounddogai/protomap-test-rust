use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a limit forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit forecast as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitForecast {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub external_ref: String,
    pub status: LimitForecastStatus,
}

impl LimitForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitForecastStatus) -> bool {
        match self.status {
            LimitForecastStatus::Requested => {
                return next == LimitForecastStatus::Granted || next == LimitForecastStatus::Expired;
            }
            LimitForecastStatus::Granted => {
                return next == LimitForecastStatus::Revoked || next == LimitForecastStatus::Expired;
            }
            LimitForecastStatus::Revoked => {
                match next {
                    LimitForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: LimitForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_volume(&self) -> f64 {
        ((self.margin + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitForecast {
        LimitForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            margin: self.margin,
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit forecast in status requested.
pub fn new_limit_forecast(code: &str, label: &str) -> LimitForecast {
    LimitForecast {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        margin: 0.0,
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        external_ref: "".to_string(),
        status: LimitForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_forecast("code-369", "label-215");
        assert_eq!(item.status, LimitForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_limit_forecast("code-369", "label-215");
        if item.move_to(LimitForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_forecast("code-369", "label-215");
        item.status = LimitForecastStatus::Expired;
        assert!(
            item.move_to(LimitForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
