use super::*;

use acme_kit::text;

/// Where a forecast baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ForecastBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ForecastBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast baseline as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaseline {
    pub code: String,
    pub name: String,
    pub label: String,
    pub source: String,
    pub issuer: String,
    pub contact_email: String,
    pub cost: f64,
    pub created_at: i64,
    pub approved: bool,
    pub timezone: String,
    pub status: ForecastBaselineStatus,
}

impl ForecastBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastBaselineStatus) -> bool {
        match self.status {
            ForecastBaselineStatus::Requested => {
                return next == ForecastBaselineStatus::Granted || next == ForecastBaselineStatus::Revoked;
            }
            ForecastBaselineStatus::Granted => {
                return next == ForecastBaselineStatus::Revoked || next == ForecastBaselineStatus::Expired;
            }
            ForecastBaselineStatus::Revoked => {
                return next == ForecastBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        (self.cost * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastBaseline {
        ForecastBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            issuer: self.issuer.clone(),
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            created_at: self.created_at,
            approved: self.approved,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a forecast baseline in status requested.
pub fn new_forecast_baseline(code: &str, name: &str) -> ForecastBaseline {
    ForecastBaseline {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        source: "".to_string(),
        issuer: "".to_string(),
        contact_email: "".to_string(),
        cost: 0.0,
        created_at: 0,
        approved: false,
        timezone: "".to_string(),
        status: ForecastBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_forecast_baseline("code-919", "name-542");
        assert_eq!(item.status, ForecastBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_forecast_baseline("code-919", "name-542");
        if item.move_to(ForecastBaselineStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastBaselineStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_forecast_baseline("code-919", "name-542");
        item.status = ForecastBaselineStatus::Expired;
        assert!(
            item.move_to(ForecastBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
