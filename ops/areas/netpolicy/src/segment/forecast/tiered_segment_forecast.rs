use super::*;

use acme_kit::text;

/// Where a tiered segment forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSegmentForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredSegmentForecastStatus {
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

impl std::fmt::Display for TieredSegmentForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered segment forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentForecast {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub tenant_id: String,
    pub minimum: f64,
    pub principal: String,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub status: TieredSegmentForecastStatus,
}

impl TieredSegmentForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSegmentForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSegmentForecastStatus) -> bool {
        match self.status {
            TieredSegmentForecastStatus::Planned => {
                return next == TieredSegmentForecastStatus::InProgress;
            }
            TieredSegmentForecastStatus::Scheduled => {
                match next {
                    TieredSegmentForecastStatus::InProgress | TieredSegmentForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSegmentForecastStatus::InProgress => {
                match next {
                    TieredSegmentForecastStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSegmentForecastStatus::Completed => {
                match next {
                    TieredSegmentForecastStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredSegmentForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered segment forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSegmentForecast {
        TieredSegmentForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            minimum: self.minimum,
            principal: self.principal.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered segment forecast in status planned.
pub fn new_tiered_segment_forecast(code: &str, title: &str) -> TieredSegmentForecast {
    TieredSegmentForecast {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        tenant_id: "".to_string(),
        minimum: 0.0,
        principal: "".to_string(),
        budget: 0.0,
        ttl_seconds: 0,
        quantity: 0,
        status: TieredSegmentForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_segment_forecast("code-736", "title-502");
        assert_eq!(item.status, TieredSegmentForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_segment_forecast("code-736", "title-502");
        if item.move_to(TieredSegmentForecastStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSegmentForecastStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_segment_forecast("code-736", "title-502");
        item.status = TieredSegmentForecastStatus::Cancelled;
        assert!(
            item.move_to(TieredSegmentForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
