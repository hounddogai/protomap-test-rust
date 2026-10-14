use super::*;

use acme_kit::text;

/// Where a health check batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl HealthCheckBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for HealthCheckBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatch {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub external_ref: String,
    pub rotation_days: i64,
    pub status: HealthCheckBatchStatus,
}

impl HealthCheckBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckBatchStatus) -> bool {
        match self.status {
            HealthCheckBatchStatus::Requested => {
                return next == HealthCheckBatchStatus::Revoked;
            }
            HealthCheckBatchStatus::Granted => {
                match next {
                    HealthCheckBatchStatus::Revoked | HealthCheckBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckBatchStatus::Revoked => {
                match next {
                    HealthCheckBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: HealthCheckBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check batch", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckBatch {
        HealthCheckBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check batch in status requested.
pub fn new_health_check_batch(code: &str, title: &str) -> HealthCheckBatch {
    HealthCheckBatch {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        external_ref: "".to_string(),
        rotation_days: 0,
        status: HealthCheckBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_health_check_batch("code-785", "title-21");
        assert_eq!(item.status, HealthCheckBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_health_check_batch("code-785", "title-21");
        if item.move_to(HealthCheckBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_health_check_batch("code-785", "title-21");
        item.status = HealthCheckBatchStatus::Expired;
        assert!(
            item.move_to(HealthCheckBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
