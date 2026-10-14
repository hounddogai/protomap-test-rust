use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleForecastStatus {
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

impl std::fmt::Display for RuleForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleForecast {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub quota_units: i64,
    pub budget: f64,
    pub amount: f64,
    pub phone: String,
    pub key_id: String,
    pub attributes: BTreeMap<String, String>,
    pub status: RuleForecastStatus,
}

impl RuleForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleForecastStatus) -> bool {
        match self.status {
            RuleForecastStatus::Open => {
                return next == RuleForecastStatus::Resolved;
            }
            RuleForecastStatus::InReview => {
                return next == RuleForecastStatus::Resolved;
            }
            RuleForecastStatus::Resolved => {
                match next {
                    RuleForecastStatus::Reopened | RuleForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleForecastStatus::Reopened => {
                return next == RuleForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule forecast {} cannot move from {} to {}",
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

    pub fn weighted_total(&self) -> f64 {
        (self.amount * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleForecast {
        RuleForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            quota_units: self.quota_units,
            budget: self.budget,
            amount: self.amount,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rule forecast in status open.
pub fn new_rule_forecast(code: &str, label: &str) -> RuleForecast {
    RuleForecast {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        quota_units: 0,
        budget: 0.0,
        amount: 0.0,
        phone: "".to_string(),
        key_id: "".to_string(),
        attributes: BTreeMap::new(),
        status: RuleForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rule_forecast("code-449", "label-834");
        assert_eq!(item.status, RuleForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rule_forecast("code-449", "label-834");
        if item.move_to(RuleForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rule_forecast("code-449", "label-834");
        item.status = RuleForecastStatus::Archived;
        assert!(item.move_to(RuleForecastStatus::Open).is_err(), "expected move to fail");
    }
}
