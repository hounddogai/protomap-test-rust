use super::*;

use acme_kit::text;

/// Where a lenient allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAllowlistForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientAllowlistForecastStatus {
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

impl std::fmt::Display for LenientAllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient allowlist forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecast {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub version: i64,
    pub scope: String,
    pub country: String,
    pub host: String,
    pub timezone: String,
    pub status: LenientAllowlistForecastStatus,
}

impl LenientAllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAllowlistForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAllowlistForecastStatus) -> bool {
        match self.status {
            LenientAllowlistForecastStatus::Idle => {
                match next {
                    LenientAllowlistForecastStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistForecastStatus::Warming => {
                match next {
                    LenientAllowlistForecastStatus::Ready | LenientAllowlistForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistForecastStatus::Ready => {
                match next {
                    LenientAllowlistForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistForecastStatus::Draining => {
                return next == LenientAllowlistForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientAllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> LenientAllowlistForecast {
        LenientAllowlistForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            version: self.version,
            scope: self.scope.clone(),
            country: self.country.clone(),
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient allowlist forecast in status idle.
pub fn new_lenient_allowlist_forecast(code: &str, title: &str) -> LenientAllowlistForecast {
    LenientAllowlistForecast {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        version: 0,
        scope: "".to_string(),
        country: "".to_string(),
        host: "".to_string(),
        timezone: "".to_string(),
        status: LenientAllowlistForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_allowlist_forecast("code-466", "title-914");
        assert_eq!(item.status, LenientAllowlistForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_allowlist_forecast("code-466", "title-914");
        if item.move_to(LenientAllowlistForecastStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAllowlistForecastStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_allowlist_forecast("code-466", "title-914");
        item.status = LenientAllowlistForecastStatus::Stopped;
        assert!(
            item.move_to(LenientAllowlistForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
