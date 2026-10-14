use super::*;

use acme_kit::text;

/// Where a seasonal ttl policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTtlPolicyEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalTtlPolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalTtlPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal ttl policy estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEstimate {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub issuer: String,
    pub budget: f64,
    pub capacity: i64,
    pub description: String,
    pub name: String,
    pub country: String,
    pub status: SeasonalTtlPolicyEstimateStatus,
}

impl SeasonalTtlPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTtlPolicyEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTtlPolicyEstimateStatus) -> bool {
        match self.status {
            SeasonalTtlPolicyEstimateStatus::Requested => {
                return next == SeasonalTtlPolicyEstimateStatus::Expired;
            }
            SeasonalTtlPolicyEstimateStatus::Granted => {
                return next == SeasonalTtlPolicyEstimateStatus::Expired;
            }
            SeasonalTtlPolicyEstimateStatus::Revoked => {
                return next == SeasonalTtlPolicyEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTtlPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal ttl policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal ttl policy estimate", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.budget * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTtlPolicyEstimate {
        SeasonalTtlPolicyEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            issuer: self.issuer.clone(),
            budget: self.budget,
            capacity: self.capacity,
            description: self.description.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal ttl policy estimate in status requested.
pub fn new_seasonal_ttl_policy_estimate(code: &str, label: &str) -> SeasonalTtlPolicyEstimate {
    SeasonalTtlPolicyEstimate {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        issuer: "".to_string(),
        budget: 0.0,
        capacity: 0,
        description: "".to_string(),
        name: "".to_string(),
        country: "".to_string(),
        status: SeasonalTtlPolicyEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_ttl_policy_estimate("code-6", "label-568");
        assert_eq!(item.status, SeasonalTtlPolicyEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_ttl_policy_estimate("code-6", "label-568");
        if item.move_to(SeasonalTtlPolicyEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTtlPolicyEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_ttl_policy_estimate("code-6", "label-568");
        item.status = SeasonalTtlPolicyEstimateStatus::Expired;
        assert!(
            item.move_to(SeasonalTtlPolicyEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
