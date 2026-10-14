use super::*;

use acme_kit::text;

/// Where a isolation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationForecastStatus {
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

impl std::fmt::Display for IsolationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationForecast {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub batch_size: i64,
    pub quota_units: i64,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub discount: f64,
    pub port: i64,
    pub name: String,
    pub source: String,
    pub status: IsolationForecastStatus,
}

impl IsolationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationForecastStatus) -> bool {
        match self.status {
            IsolationForecastStatus::Idle => {
                match next {
                    IsolationForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationForecastStatus::Warming => {
                match next {
                    IsolationForecastStatus::Draining | IsolationForecastStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationForecastStatus::Ready => {
                return next == IsolationForecastStatus::Draining || next == IsolationForecastStatus::Stopped;
            }
            IsolationForecastStatus::Draining => {
                return next == IsolationForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation forecast {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.discount * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationForecast {
        IsolationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            discount: self.discount,
            port: self.port,
            name: self.name.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation forecast in status idle.
pub fn new_isolation_forecast(code: &str, title: &str) -> IsolationForecast {
    IsolationForecast {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        batch_size: 0,
        quota_units: 0,
        archived: false,
        ttl_seconds: 0,
        discount: 0.0,
        port: 0,
        name: "".to_string(),
        source: "".to_string(),
        status: IsolationForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_forecast("code-473", "title-647");
        assert_eq!(item.status, IsolationForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_isolation_forecast("code-473", "title-647");
        if item.move_to(IsolationForecastStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationForecastStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_forecast("code-473", "title-647");
        item.status = IsolationForecastStatus::Stopped;
        assert!(
            item.move_to(IsolationForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
