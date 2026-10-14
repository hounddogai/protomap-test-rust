use super::*;

use acme_kit::text;

/// Where a seasonal isolation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIsolationEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalIsolationEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalIsolationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal isolation estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimate {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub created_at: i64,
    pub cost: f64,
    pub description: String,
    pub title: String,
    pub region: String,
    pub status: SeasonalIsolationEstimateStatus,
}

impl SeasonalIsolationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIsolationEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIsolationEstimateStatus) -> bool {
        match self.status {
            SeasonalIsolationEstimateStatus::Requested => {
                return next == SeasonalIsolationEstimateStatus::Granted
                    || next == SeasonalIsolationEstimateStatus::Expired;
            }
            SeasonalIsolationEstimateStatus::Granted => {
                return next == SeasonalIsolationEstimateStatus::Revoked;
            }
            SeasonalIsolationEstimateStatus::Revoked => {
                return next == SeasonalIsolationEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalIsolationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal isolation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.cost + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIsolationEstimate {
        SeasonalIsolationEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            created_at: self.created_at,
            cost: self.cost,
            description: self.description.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a seasonal isolation estimate in status requested.
pub fn new_seasonal_isolation_estimate(code: &str, label: &str) -> SeasonalIsolationEstimate {
    SeasonalIsolationEstimate {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        created_at: 0,
        cost: 0.0,
        description: "".to_string(),
        title: "".to_string(),
        region: "".to_string(),
        status: SeasonalIsolationEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_isolation_estimate("code-685", "label-806");
        assert_eq!(item.status, SeasonalIsolationEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_isolation_estimate("code-685", "label-806");
        if item.move_to(SeasonalIsolationEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalIsolationEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_isolation_estimate("code-685", "label-806");
        item.status = SeasonalIsolationEstimateStatus::Expired;
        assert!(
            item.move_to(SeasonalIsolationEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
