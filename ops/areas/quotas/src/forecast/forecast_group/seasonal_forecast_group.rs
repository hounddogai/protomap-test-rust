use super::*;

use acme_kit::text;

/// Where a seasonal forecast group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalForecastGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalForecastGroupStatus {
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

impl std::fmt::Display for SeasonalForecastGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal forecast group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastGroup {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub duration_minutes: i64,
    pub namespace: String,
    pub segment: String,
    pub checksum: String,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub host: String,
    pub status: SeasonalForecastGroupStatus,
}

impl SeasonalForecastGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalForecastGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalForecastGroupStatus) -> bool {
        match self.status {
            SeasonalForecastGroupStatus::New => {
                match next {
                    SeasonalForecastGroupStatus::Done | SeasonalForecastGroupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastGroupStatus::Queued => {
                return next == SeasonalForecastGroupStatus::Done;
            }
            SeasonalForecastGroupStatus::Running => {
                return next == SeasonalForecastGroupStatus::Failed;
            }
            SeasonalForecastGroupStatus::Done => {
                return next == SeasonalForecastGroupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalForecastGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal forecast group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal forecast group", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalForecastGroup {
        SeasonalForecastGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a seasonal forecast group in status new.
pub fn new_seasonal_forecast_group(code: &str, label: &str) -> SeasonalForecastGroup {
    SeasonalForecastGroup {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        duration_minutes: 0,
        namespace: "".to_string(),
        segment: "".to_string(),
        checksum: "".to_string(),
        ttl_seconds: 0,
        quantity: 0,
        host: "".to_string(),
        status: SeasonalForecastGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_forecast_group("code-93", "label-421");
        assert_eq!(item.status, SeasonalForecastGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_seasonal_forecast_group("code-93", "label-421");
        if item.move_to(SeasonalForecastGroupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalForecastGroupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_forecast_group("code-93", "label-421");
        item.status = SeasonalForecastGroupStatus::Failed;
        assert!(
            item.move_to(SeasonalForecastGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
