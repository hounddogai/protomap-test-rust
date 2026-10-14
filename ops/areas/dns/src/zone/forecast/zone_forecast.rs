use super::*;

use acme_kit::text;

/// Where a zone forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneForecastStatus {
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

impl std::fmt::Display for ZoneForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneForecast {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub namespace: String,
    pub capacity: i64,
    pub sequence: i64,
    pub minimum: f64,
    pub source: String,
    pub status: ZoneForecastStatus,
}

impl ZoneForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneForecastStatus) -> bool {
        match self.status {
            ZoneForecastStatus::Planned => {
                return next == ZoneForecastStatus::InProgress;
            }
            ZoneForecastStatus::Scheduled => {
                match next {
                    ZoneForecastStatus::InProgress | ZoneForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneForecastStatus::InProgress => {
                return next == ZoneForecastStatus::Cancelled;
            }
            ZoneForecastStatus::Completed => {
                return next == ZoneForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone forecast", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneForecast {
        ZoneForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            capacity: self.capacity,
            sequence: self.sequence,
            minimum: self.minimum,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a zone forecast in status planned.
pub fn new_zone_forecast(code: &str, name: &str) -> ZoneForecast {
    ZoneForecast {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        namespace: "".to_string(),
        capacity: 0,
        sequence: 0,
        minimum: 0.0,
        source: "".to_string(),
        status: ZoneForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_forecast("code-131", "name-397");
        assert_eq!(item.status, ZoneForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_zone_forecast("code-131", "name-397");
        if item.move_to(ZoneForecastStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneForecastStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_forecast("code-131", "name-397");
        item.status = ZoneForecastStatus::Cancelled;
        assert!(
            item.move_to(ZoneForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
