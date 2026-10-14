use super::*;

use acme_kit::text;

/// Where a zone estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEstimate {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub currency: String,
    pub sequence: i64,
    pub window_days: i64,
    pub budget: f64,
    pub owner_id: String,
    pub threshold: f64,
    pub name: String,
    pub status: ZoneEstimateStatus,
}

impl ZoneEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneEstimateStatus) -> bool {
        match self.status {
            ZoneEstimateStatus::Created => {
                match next {
                    ZoneEstimateStatus::Validated | ZoneEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneEstimateStatus::Validated => {
                return next == ZoneEstimateStatus::Published || next == ZoneEstimateStatus::Retired;
            }
            ZoneEstimateStatus::Published => {
                match next {
                    ZoneEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: ZoneEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone estimate", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.sequence as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneEstimate {
        ZoneEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            currency: self.currency.clone(),
            sequence: self.sequence,
            window_days: self.window_days,
            budget: self.budget,
            owner_id: self.owner_id.clone(),
            threshold: self.threshold,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone estimate in status created.
pub fn new_zone_estimate(code: &str, title: &str) -> ZoneEstimate {
    ZoneEstimate {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        currency: "".to_string(),
        sequence: 0,
        window_days: 0,
        budget: 0.0,
        owner_id: "".to_string(),
        threshold: 0.0,
        name: "".to_string(),
        status: ZoneEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_estimate("code-329", "title-215");
        assert_eq!(item.status, ZoneEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_zone_estimate("code-329", "title-215");
        if item.move_to(ZoneEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_estimate("code-329", "title-215");
        item.status = ZoneEstimateStatus::Retired;
        assert!(
            item.move_to(ZoneEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
