use super::*;

use acme_kit::text;

/// Where a policy forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PolicyForecastStatus {
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

impl std::fmt::Display for PolicyForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyForecast {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub approved: bool,
    pub contact_email: String,
    pub weight: f64,
    pub active: bool,
    pub margin: f64,
    pub cipher: String,
    pub status: PolicyForecastStatus,
}

impl PolicyForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyForecastStatus) -> bool {
        match self.status {
            PolicyForecastStatus::Idle => {
                return next == PolicyForecastStatus::Ready || next == PolicyForecastStatus::Draining;
            }
            PolicyForecastStatus::Warming => {
                return next == PolicyForecastStatus::Draining;
            }
            PolicyForecastStatus::Ready => {
                return next == PolicyForecastStatus::Stopped;
            }
            PolicyForecastStatus::Draining => {
                return next == PolicyForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy forecast {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyForecast {
        PolicyForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            weight: self.weight,
            active: self.active,
            margin: self.margin,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy forecast in status idle.
pub fn new_policy_forecast(code: &str, title: &str) -> PolicyForecast {
    PolicyForecast {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        approved: false,
        contact_email: "".to_string(),
        weight: 0.0,
        active: false,
        margin: 0.0,
        cipher: "".to_string(),
        status: PolicyForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_policy_forecast("code-370", "title-488");
        assert_eq!(item.status, PolicyForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_policy_forecast("code-370", "title-488");
        if item.move_to(PolicyForecastStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyForecastStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_policy_forecast("code-370", "title-488");
        item.status = PolicyForecastStatus::Stopped;
        assert!(
            item.move_to(PolicyForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
