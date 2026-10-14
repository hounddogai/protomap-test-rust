use super::*;

use acme_kit::text;

/// Where a measurement estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MeasurementEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MeasurementEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimate {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub window_days: i64,
    pub category: String,
    pub expires_at: i64,
    pub status: MeasurementEstimateStatus,
}

impl MeasurementEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementEstimateStatus) -> bool {
        match self.status {
            MeasurementEstimateStatus::Requested => {
                match next {
                    MeasurementEstimateStatus::Granted | MeasurementEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementEstimateStatus::Granted => {
                return next == MeasurementEstimateStatus::Revoked;
            }
            MeasurementEstimateStatus::Revoked => {
                return next == MeasurementEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement estimate", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementEstimate {
        MeasurementEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            category: self.category.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement estimate in status requested.
pub fn new_measurement_estimate(code: &str, name: &str) -> MeasurementEstimate {
    MeasurementEstimate {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        window_days: 0,
        category: "".to_string(),
        expires_at: 0,
        status: MeasurementEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_measurement_estimate("code-921", "name-467");
        assert_eq!(item.status, MeasurementEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_measurement_estimate("code-921", "name-467");
        if item.move_to(MeasurementEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_measurement_estimate("code-921", "name-467");
        item.status = MeasurementEstimateStatus::Expired;
        assert!(
            item.move_to(MeasurementEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
