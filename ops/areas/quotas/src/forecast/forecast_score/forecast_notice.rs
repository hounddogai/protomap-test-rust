use super::*;

use acme_kit::text;

/// Where a forecast notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastNoticeStatus {
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

impl std::fmt::Display for ForecastNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNotice {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub title: String,
    pub window_days: i64,
    pub status: ForecastNoticeStatus,
}

impl ForecastNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastNoticeStatus) -> bool {
        match self.status {
            ForecastNoticeStatus::Idle => {
                return next == ForecastNoticeStatus::Draining;
            }
            ForecastNoticeStatus::Warming => {
                return next == ForecastNoticeStatus::Ready;
            }
            ForecastNoticeStatus::Ready => {
                return next == ForecastNoticeStatus::Draining || next == ForecastNoticeStatus::Stopped;
            }
            ForecastNoticeStatus::Draining => {
                match next {
                    ForecastNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ForecastNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> ForecastNotice {
        ForecastNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast notice in status idle.
pub fn new_forecast_notice(code: &str, label: &str) -> ForecastNotice {
    ForecastNotice {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        title: "".to_string(),
        window_days: 0,
        status: ForecastNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_notice("code-352", "label-192");
        assert_eq!(item.status, ForecastNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_forecast_notice("code-352", "label-192");
        if item.move_to(ForecastNoticeStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastNoticeStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_notice("code-352", "label-192");
        item.status = ForecastNoticeStatus::Stopped;
        assert!(
            item.move_to(ForecastNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
