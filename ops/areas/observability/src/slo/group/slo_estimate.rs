use super::*;

use acme_kit::text;

/// Where a slo estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SloEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SloEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloEstimate {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub minimum: f64,
    pub starts_at: i64,
    pub discount: f64,
    pub reviewer: String,
    pub grace_days: i64,
    pub status: SloEstimateStatus,
}

impl SloEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloEstimateStatus) -> bool {
        match self.status {
            SloEstimateStatus::Created => {
                match next {
                    SloEstimateStatus::Published | SloEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloEstimateStatus::Validated => {
                match next {
                    SloEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloEstimateStatus::Published => {
                return next == SloEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.amount - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloEstimate {
        SloEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            minimum: self.minimum,
            starts_at: self.starts_at,
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo estimate in status created.
pub fn new_slo_estimate(code: &str, name: &str) -> SloEstimate {
    SloEstimate {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        minimum: 0.0,
        starts_at: 0,
        discount: 0.0,
        reviewer: "".to_string(),
        grace_days: 0,
        status: SloEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_slo_estimate("code-26", "name-839");
        assert_eq!(item.status, SloEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_slo_estimate("code-26", "name-839");
        if item.move_to(SloEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_slo_estimate("code-26", "name-839");
        item.status = SloEstimateStatus::Retired;
        assert!(
            item.move_to(SloEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
