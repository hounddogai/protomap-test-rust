use super::*;

use acme_kit::text;

/// Where a pending delegation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDelegationForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingDelegationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingDelegationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending delegation forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationForecast {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub retries: i64,
    pub budget: f64,
    pub reviewer: String,
    pub updated_at: i64,
    pub revision: i64,
    pub scope: String,
    pub status: PendingDelegationForecastStatus,
}

impl PendingDelegationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDelegationForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDelegationForecastStatus) -> bool {
        match self.status {
            PendingDelegationForecastStatus::Requested => {
                return next == PendingDelegationForecastStatus::Granted;
            }
            PendingDelegationForecastStatus::Granted => {
                return next == PendingDelegationForecastStatus::Expired;
            }
            PendingDelegationForecastStatus::Revoked => {
                match next {
                    PendingDelegationForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingDelegationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending delegation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending delegation forecast", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDelegationForecast {
        PendingDelegationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            budget: self.budget,
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            revision: self.revision,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending delegation forecast in status requested.
pub fn new_pending_delegation_forecast(code: &str, label: &str) -> PendingDelegationForecast {
    PendingDelegationForecast {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        retries: 0,
        budget: 0.0,
        reviewer: "".to_string(),
        updated_at: 0,
        revision: 0,
        scope: "".to_string(),
        status: PendingDelegationForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_delegation_forecast("code-970", "label-30");
        assert_eq!(item.status, PendingDelegationForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_delegation_forecast("code-970", "label-30");
        if item.move_to(PendingDelegationForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDelegationForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_delegation_forecast("code-970", "label-30");
        item.status = PendingDelegationForecastStatus::Expired;
        assert!(
            item.move_to(PendingDelegationForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
