use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient reservation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientReservationForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientReservationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LenientReservationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient reservation forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecast {
    pub code: String,
    pub name: String,
    pub category: String,
    pub rotation_days: i64,
    pub audience: String,
    pub archived: bool,
    pub description: String,
    pub attributes: BTreeMap<String, String>,
    pub status: LenientReservationForecastStatus,
}

impl LenientReservationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientReservationForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientReservationForecastStatus) -> bool {
        match self.status {
            LenientReservationForecastStatus::Idle => {
                return next == LenientReservationForecastStatus::Ready;
            }
            LenientReservationForecastStatus::Warming => {
                return next == LenientReservationForecastStatus::Draining
                    || next == LenientReservationForecastStatus::Stopped;
            }
            LenientReservationForecastStatus::Ready => {
                return next == LenientReservationForecastStatus::Stopped;
            }
            LenientReservationForecastStatus::Draining => {
                match next {
                    LenientReservationForecastStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientReservationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient reservation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> LenientReservationForecast {
        LenientReservationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            audience: self.audience.clone(),
            archived: self.archived,
            description: self.description.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient reservation forecast in status idle.
pub fn new_lenient_reservation_forecast(code: &str, name: &str) -> LenientReservationForecast {
    LenientReservationForecast {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        rotation_days: 0,
        audience: "".to_string(),
        archived: false,
        description: "".to_string(),
        attributes: BTreeMap::new(),
        status: LenientReservationForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_reservation_forecast("code-937", "name-644");
        assert_eq!(item.status, LenientReservationForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_reservation_forecast("code-937", "name-644");
        if item.move_to(LenientReservationForecastStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientReservationForecastStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_reservation_forecast("code-937", "name-644");
        item.status = LenientReservationForecastStatus::Stopped;
        assert!(
            item.move_to(LenientReservationForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
