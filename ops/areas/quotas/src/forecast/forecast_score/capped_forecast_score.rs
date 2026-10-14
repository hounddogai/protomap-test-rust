use super::*;

use acme_kit::text;

/// Where a capped forecast score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedForecastScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedForecastScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedForecastScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped forecast score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScore {
    pub code: String,
    pub name: String,
    pub host: String,
    pub notes: String,
    pub tenant_id: String,
    pub discount: f64,
    pub grace_days: i64,
    pub version: i64,
    pub capacity: i64,
    pub status: CappedForecastScoreStatus,
}

impl CappedForecastScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedForecastScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedForecastScoreStatus) -> bool {
        match self.status {
            CappedForecastScoreStatus::Proposed => {
                return next == CappedForecastScoreStatus::Rejected || next == CappedForecastScoreStatus::Withdrawn;
            }
            CappedForecastScoreStatus::Approved => {
                return next == CappedForecastScoreStatus::Rejected || next == CappedForecastScoreStatus::Withdrawn;
            }
            CappedForecastScoreStatus::Rejected => {
                match next {
                    CappedForecastScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedForecastScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped forecast score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped forecast score", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.capacity as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedForecastScore {
        CappedForecastScore {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            discount: self.discount,
            grace_days: self.grace_days,
            version: self.version,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped forecast score in status proposed.
pub fn new_capped_forecast_score(code: &str, name: &str) -> CappedForecastScore {
    CappedForecastScore {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        discount: 0.0,
        grace_days: 0,
        version: 0,
        capacity: 0,
        status: CappedForecastScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_forecast_score("code-484", "name-79");
        assert_eq!(item.status, CappedForecastScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_capped_forecast_score("code-484", "name-79");
        if item.move_to(CappedForecastScoreStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedForecastScoreStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_forecast_score("code-484", "name-79");
        item.status = CappedForecastScoreStatus::Withdrawn;
        assert!(
            item.move_to(CappedForecastScoreStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
