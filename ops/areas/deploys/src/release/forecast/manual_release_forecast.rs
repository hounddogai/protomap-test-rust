use super::*;

use acme_kit::text;

/// Where a manual release forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualReleaseForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualReleaseForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualReleaseForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual release forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecast {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub summary: String,
    pub country: String,
    pub status: ManualReleaseForecastStatus,
}

impl ManualReleaseForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualReleaseForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualReleaseForecastStatus) -> bool {
        match self.status {
            ManualReleaseForecastStatus::New => {
                match next {
                    ManualReleaseForecastStatus::Queued | ManualReleaseForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualReleaseForecastStatus::Queued => {
                return next == ManualReleaseForecastStatus::Running;
            }
            ManualReleaseForecastStatus::Running => {
                match next {
                    ManualReleaseForecastStatus::Done | ManualReleaseForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualReleaseForecastStatus::Done => {
                return next == ManualReleaseForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualReleaseForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual release forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ManualReleaseForecast {
        ManualReleaseForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual release forecast in status new.
pub fn new_manual_release_forecast(code: &str, name: &str) -> ManualReleaseForecast {
    ManualReleaseForecast {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        summary: "".to_string(),
        country: "".to_string(),
        status: ManualReleaseForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_release_forecast("code-649", "name-401");
        assert_eq!(item.status, ManualReleaseForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_release_forecast("code-649", "name-401");
        if item.move_to(ManualReleaseForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualReleaseForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_release_forecast("code-649", "name-401");
        item.status = ManualReleaseForecastStatus::Failed;
        assert!(
            item.move_to(ManualReleaseForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
