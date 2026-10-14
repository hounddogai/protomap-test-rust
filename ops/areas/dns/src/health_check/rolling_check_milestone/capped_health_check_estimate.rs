use super::*;

use acme_kit::text;

/// Where a capped health check estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedHealthCheckEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedHealthCheckEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedHealthCheckEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped health check estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedHealthCheckEstimate {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub priority: i64,
    pub attempts: i64,
    pub limit: i64,
    pub timezone: String,
    pub status: CappedHealthCheckEstimateStatus,
}

impl CappedHealthCheckEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedHealthCheckEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedHealthCheckEstimateStatus) -> bool {
        match self.status {
            CappedHealthCheckEstimateStatus::Created => {
                match next {
                    CappedHealthCheckEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedHealthCheckEstimateStatus::Validated => {
                return next == CappedHealthCheckEstimateStatus::Retired;
            }
            CappedHealthCheckEstimateStatus::Published => {
                match next {
                    CappedHealthCheckEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedHealthCheckEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped health check estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.rate * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedHealthCheckEstimate {
        CappedHealthCheckEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            priority: self.priority,
            attempts: self.attempts,
            limit: self.limit,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped health check estimate in status created.
pub fn new_capped_health_check_estimate(code: &str, title: &str) -> CappedHealthCheckEstimate {
    CappedHealthCheckEstimate {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        priority: 0,
        attempts: 0,
        limit: 0,
        timezone: "".to_string(),
        status: CappedHealthCheckEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_health_check_estimate("code-661", "title-543");
        assert_eq!(item.status, CappedHealthCheckEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_health_check_estimate("code-661", "title-543");
        if item.move_to(CappedHealthCheckEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedHealthCheckEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_health_check_estimate("code-661", "title-543");
        item.status = CappedHealthCheckEstimateStatus::Retired;
        assert!(
            item.move_to(CappedHealthCheckEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
