use super::*;

use acme_kit::text;

/// Where a weighted identity forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedIdentityForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedIdentityForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for WeightedIdentityForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted identity forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecast {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub ceiling: f64,
    pub active: bool,
    pub source: String,
    pub created_at: i64,
    pub status: WeightedIdentityForecastStatus,
}

impl WeightedIdentityForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedIdentityForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedIdentityForecastStatus) -> bool {
        match self.status {
            WeightedIdentityForecastStatus::Planned => {
                match next {
                    WeightedIdentityForecastStatus::Scheduled | WeightedIdentityForecastStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityForecastStatus::Scheduled => {
                return next == WeightedIdentityForecastStatus::InProgress
                    || next == WeightedIdentityForecastStatus::Cancelled;
            }
            WeightedIdentityForecastStatus::InProgress => {
                match next {
                    WeightedIdentityForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityForecastStatus::Completed => {
                match next {
                    WeightedIdentityForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedIdentityForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted identity forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted identity forecast", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ends_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedIdentityForecast {
        WeightedIdentityForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            active: self.active,
            source: self.source.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted identity forecast in status planned.
pub fn new_weighted_identity_forecast(code: &str, title: &str) -> WeightedIdentityForecast {
    WeightedIdentityForecast {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        ceiling: 0.0,
        active: false,
        source: "".to_string(),
        created_at: 0,
        status: WeightedIdentityForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_identity_forecast("code-832", "title-202");
        assert_eq!(item.status, WeightedIdentityForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_weighted_identity_forecast("code-832", "title-202");
        if item.move_to(WeightedIdentityForecastStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedIdentityForecastStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_identity_forecast("code-832", "title-202");
        item.status = WeightedIdentityForecastStatus::Cancelled;
        assert!(
            item.move_to(WeightedIdentityForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
