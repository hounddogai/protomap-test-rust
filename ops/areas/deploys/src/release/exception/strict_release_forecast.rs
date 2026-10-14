use super::*;

use acme_kit::text;

/// Where a strict release forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReleaseForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictReleaseForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictReleaseForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict release forecast as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseForecast {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub updated_at: i64,
    pub namespace: String,
    pub weight: f64,
    pub status: StrictReleaseForecastStatus,
}

impl StrictReleaseForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReleaseForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReleaseForecastStatus) -> bool {
        match self.status {
            StrictReleaseForecastStatus::Created => {
                return next == StrictReleaseForecastStatus::Published;
            }
            StrictReleaseForecastStatus::Validated => {
                match next {
                    StrictReleaseForecastStatus::Published | StrictReleaseForecastStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseForecastStatus::Published => {
                match next {
                    StrictReleaseForecastStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictReleaseForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict release forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.budget * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReleaseForecast {
        StrictReleaseForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict release forecast in status created.
pub fn new_strict_release_forecast(code: &str, name: &str) -> StrictReleaseForecast {
    StrictReleaseForecast {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        updated_at: 0,
        namespace: "".to_string(),
        weight: 0.0,
        status: StrictReleaseForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_release_forecast("code-489", "name-219");
        assert_eq!(item.status, StrictReleaseForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_strict_release_forecast("code-489", "name-219");
        if item.move_to(StrictReleaseForecastStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictReleaseForecastStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_release_forecast("code-489", "name-219");
        item.status = StrictReleaseForecastStatus::Retired;
        assert!(
            item.move_to(StrictReleaseForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
