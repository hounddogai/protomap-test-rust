use super::*;

use acme_kit::text;

/// Where a capped evidence forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEvidenceForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedEvidenceForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedEvidenceForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped evidence forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceForecast {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub version: i64,
    pub ends_at: i64,
    pub category: String,
    pub status: CappedEvidenceForecastStatus,
}

impl CappedEvidenceForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEvidenceForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEvidenceForecastStatus) -> bool {
        match self.status {
            CappedEvidenceForecastStatus::Proposed => {
                match next {
                    CappedEvidenceForecastStatus::Approved | CappedEvidenceForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEvidenceForecastStatus::Approved => {
                match next {
                    CappedEvidenceForecastStatus::Rejected | CappedEvidenceForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEvidenceForecastStatus::Rejected => {
                match next {
                    CappedEvidenceForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedEvidenceForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped evidence forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ends_at as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEvidenceForecast {
        CappedEvidenceForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            version: self.version,
            ends_at: self.ends_at,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped evidence forecast in status proposed.
pub fn new_capped_evidence_forecast(code: &str, name: &str) -> CappedEvidenceForecast {
    CappedEvidenceForecast {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        version: 0,
        ends_at: 0,
        category: "".to_string(),
        status: CappedEvidenceForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_evidence_forecast("code-506", "name-918");
        assert_eq!(item.status, CappedEvidenceForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_evidence_forecast("code-506", "name-918");
        if item.move_to(CappedEvidenceForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEvidenceForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_evidence_forecast("code-506", "name-918");
        item.status = CappedEvidenceForecastStatus::Withdrawn;
        assert!(
            item.move_to(CappedEvidenceForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
