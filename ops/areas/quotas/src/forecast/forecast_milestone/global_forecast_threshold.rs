use super::*;

use acme_kit::text;

/// Where a global forecast threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalForecastThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalForecastThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalForecastThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global forecast threshold as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThreshold {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub score: f64,
    pub category: String,
    pub external_ref: String,
    pub visible: bool,
    pub capacity: i64,
    pub status: GlobalForecastThresholdStatus,
}

impl GlobalForecastThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalForecastThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalForecastThresholdStatus) -> bool {
        match self.status {
            GlobalForecastThresholdStatus::Created => {
                return next == GlobalForecastThresholdStatus::Validated;
            }
            GlobalForecastThresholdStatus::Validated => {
                match next {
                    GlobalForecastThresholdStatus::Published | GlobalForecastThresholdStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastThresholdStatus::Published => {
                return next == GlobalForecastThresholdStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalForecastThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global forecast threshold {} cannot move from {} to {}",
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
        format!("{} {}", "global forecast threshold", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.capacity as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalForecastThreshold {
        GlobalForecastThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            score: self.score,
            category: self.category.clone(),
            external_ref: self.external_ref.clone(),
            visible: self.visible,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global forecast threshold in status created.
pub fn new_global_forecast_threshold(code: &str, title: &str) -> GlobalForecastThreshold {
    GlobalForecastThreshold {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        score: 0.0,
        category: "".to_string(),
        external_ref: "".to_string(),
        visible: false,
        capacity: 0,
        status: GlobalForecastThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_forecast_threshold("code-610", "title-257");
        assert_eq!(item.status, GlobalForecastThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_global_forecast_threshold("code-610", "title-257");
        if item.move_to(GlobalForecastThresholdStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalForecastThresholdStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_forecast_threshold("code-610", "title-257");
        item.status = GlobalForecastThresholdStatus::Retired;
        assert!(
            item.move_to(GlobalForecastThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
