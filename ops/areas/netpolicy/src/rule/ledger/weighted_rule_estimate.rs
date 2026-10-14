use super::*;

use acme_kit::text;

/// Where a weighted rule estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedRuleEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedRuleEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted rule estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleEstimate {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub rate: f64,
    pub active: bool,
    pub margin: f64,
    pub status: WeightedRuleEstimateStatus,
}

impl WeightedRuleEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleEstimateStatus) -> bool {
        match self.status {
            WeightedRuleEstimateStatus::Requested => {
                match next {
                    WeightedRuleEstimateStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleEstimateStatus::Granted => {
                match next {
                    WeightedRuleEstimateStatus::Revoked | WeightedRuleEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleEstimateStatus::Revoked => {
                match next {
                    WeightedRuleEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedRuleEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rate - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRuleEstimate {
        WeightedRuleEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            rate: self.rate,
            active: self.active,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted rule estimate in status requested.
pub fn new_weighted_rule_estimate(code: &str, title: &str) -> WeightedRuleEstimate {
    WeightedRuleEstimate {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        rate: 0.0,
        active: false,
        margin: 0.0,
        status: WeightedRuleEstimateStatus::Requested,
    }
}
