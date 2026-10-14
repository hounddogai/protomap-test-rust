use super::*;

use acme_kit::text;

/// Where a trust domain forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TrustDomainForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainForecast {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub ends_at: i64,
    pub score: f64,
    pub capacity: i64,
    pub channel: String,
    pub burst: i64,
    pub port: i64,
    pub duration_minutes: i64,
    pub country: String,
    pub status: TrustDomainForecastStatus,
}

impl TrustDomainForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainForecastStatus) -> bool {
        match self.status {
            TrustDomainForecastStatus::Draft => {
                return next == TrustDomainForecastStatus::Active;
            }
            TrustDomainForecastStatus::Pending => {
                return next == TrustDomainForecastStatus::Active || next == TrustDomainForecastStatus::Closed;
            }
            TrustDomainForecastStatus::Active => {
                return next == TrustDomainForecastStatus::Suspended;
            }
            TrustDomainForecastStatus::Suspended => {
                return next == TrustDomainForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain forecast {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainForecast {
        TrustDomainForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            score: self.score,
            capacity: self.capacity,
            channel: self.channel.clone(),
            burst: self.burst,
            port: self.port,
            duration_minutes: self.duration_minutes,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a trust domain forecast in status draft.
pub fn new_trust_domain_forecast(code: &str, name: &str) -> TrustDomainForecast {
    TrustDomainForecast {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        ends_at: 0,
        score: 0.0,
        capacity: 0,
        channel: "".to_string(),
        burst: 0,
        port: 0,
        duration_minutes: 0,
        country: "".to_string(),
        status: TrustDomainForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_forecast("code-479", "name-683");
        assert_eq!(item.status, TrustDomainForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_trust_domain_forecast("code-479", "name-683");
        if item.move_to(TrustDomainForecastStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainForecastStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_forecast("code-479", "name-683");
        item.status = TrustDomainForecastStatus::Closed;
        assert!(
            item.move_to(TrustDomainForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
