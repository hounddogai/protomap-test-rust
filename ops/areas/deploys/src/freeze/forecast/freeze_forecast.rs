use super::*;

use acme_kit::text;

/// Where a freeze forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FreezeForecastStatus {
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

impl std::fmt::Display for FreezeForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeForecast {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub contact_email: String,
    pub segment: String,
    pub rate: f64,
    pub region: String,
    pub quantity: i64,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub currency: String,
    pub status: FreezeForecastStatus,
}

impl FreezeForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeForecastStatus) -> bool {
        match self.status {
            FreezeForecastStatus::New => {
                match next {
                    FreezeForecastStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeForecastStatus::Queued => {
                return next == FreezeForecastStatus::Done;
            }
            FreezeForecastStatus::Running => {
                match next {
                    FreezeForecastStatus::Done | FreezeForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeForecastStatus::Done => {
                return next == FreezeForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze forecast", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeForecast {
        FreezeForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            rate: self.rate,
            region: self.region.clone(),
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a freeze forecast in status new.
pub fn new_freeze_forecast(code: &str, title: &str) -> FreezeForecast {
    FreezeForecast {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        contact_email: "".to_string(),
        segment: "".to_string(),
        rate: 0.0,
        region: "".to_string(),
        quantity: 0,
        rotation_days: 0,
        duration_minutes: 0,
        currency: "".to_string(),
        status: FreezeForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_freeze_forecast("code-771", "title-469");
        assert_eq!(item.status, FreezeForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_freeze_forecast("code-771", "title-469");
        if item.move_to(FreezeForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_freeze_forecast("code-771", "title-469");
        item.status = FreezeForecastStatus::Failed;
        assert!(
            item.move_to(FreezeForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
