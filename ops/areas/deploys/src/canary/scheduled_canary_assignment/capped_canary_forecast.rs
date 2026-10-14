use super::*;

use acme_kit::text;

/// Where a capped canary forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedCanaryForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedCanaryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedCanaryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped canary forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryForecast {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub duration_minutes: i64,
    pub approved: bool,
    pub external_ref: String,
    pub summary: String,
    pub revision: i64,
    pub floor_value: f64,
    pub status: CappedCanaryForecastStatus,
}

impl CappedCanaryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedCanaryForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedCanaryForecastStatus) -> bool {
        match self.status {
            CappedCanaryForecastStatus::Created => {
                return next == CappedCanaryForecastStatus::Validated;
            }
            CappedCanaryForecastStatus::Validated => {
                return next == CappedCanaryForecastStatus::Published || next == CappedCanaryForecastStatus::Retired;
            }
            CappedCanaryForecastStatus::Published => {
                return next == CappedCanaryForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedCanaryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped canary forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedCanaryForecast {
        CappedCanaryForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            duration_minutes: self.duration_minutes,
            approved: self.approved,
            external_ref: self.external_ref.clone(),
            summary: self.summary.clone(),
            revision: self.revision,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped canary forecast in status created.
pub fn new_capped_canary_forecast(code: &str, title: &str) -> CappedCanaryForecast {
    CappedCanaryForecast {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        duration_minutes: 0,
        approved: false,
        external_ref: "".to_string(),
        summary: "".to_string(),
        revision: 0,
        floor_value: 0.0,
        status: CappedCanaryForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_canary_forecast("code-715", "title-940");
        assert_eq!(item.status, CappedCanaryForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_canary_forecast("code-715", "title-940");
        if item.move_to(CappedCanaryForecastStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedCanaryForecastStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_canary_forecast("code-715", "title-940");
        item.status = CappedCanaryForecastStatus::Retired;
        assert!(
            item.move_to(CappedCanaryForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
