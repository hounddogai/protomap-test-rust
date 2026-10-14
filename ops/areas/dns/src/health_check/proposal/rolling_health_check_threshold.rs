use super::*;

use acme_kit::text;

/// Where a rolling health check threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingHealthCheckThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingHealthCheckThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingHealthCheckThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling health check threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThreshold {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub rotation_days: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub status: RollingHealthCheckThresholdStatus,
}

impl RollingHealthCheckThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingHealthCheckThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingHealthCheckThresholdStatus) -> bool {
        match self.status {
            RollingHealthCheckThresholdStatus::Proposed => {
                return next == RollingHealthCheckThresholdStatus::Approved
                    || next == RollingHealthCheckThresholdStatus::Rejected;
            }
            RollingHealthCheckThresholdStatus::Approved => {
                return next == RollingHealthCheckThresholdStatus::Rejected;
            }
            RollingHealthCheckThresholdStatus::Rejected => {
                match next {
                    RollingHealthCheckThresholdStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingHealthCheckThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling health check threshold {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.created_at as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingHealthCheckThreshold {
        RollingHealthCheckThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling health check threshold in status proposed.
pub fn new_rolling_health_check_threshold(code: &str, title: &str) -> RollingHealthCheckThreshold {
    RollingHealthCheckThreshold {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        rotation_days: 0,
        sequence: 0,
        created_at: 0,
        status: RollingHealthCheckThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_health_check_threshold("code-992", "title-244");
        assert_eq!(item.status, RollingHealthCheckThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_health_check_threshold("code-992", "title-244");
        if item.move_to(RollingHealthCheckThresholdStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingHealthCheckThresholdStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_health_check_threshold("code-992", "title-244");
        item.status = RollingHealthCheckThresholdStatus::Withdrawn;
        assert!(
            item.move_to(RollingHealthCheckThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
