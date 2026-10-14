use super::*;

use acme_kit::text;

/// Where a escalation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EscalationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EscalationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationForecast {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub currency: String,
    pub timezone: String,
    pub notes: String,
    pub burst: i64,
    pub description: String,
    pub cost: f64,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub status: EscalationForecastStatus,
}

impl EscalationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationForecastStatus) -> bool {
        match self.status {
            EscalationForecastStatus::Proposed => {
                match next {
                    EscalationForecastStatus::Rejected | EscalationForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationForecastStatus::Approved => {
                match next {
                    EscalationForecastStatus::Rejected | EscalationForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationForecastStatus::Rejected => {
                match next {
                    EscalationForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EscalationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation forecast {} cannot move from {} to {}",
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
        format!("{} {}", "escalation forecast", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationForecast {
        EscalationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            currency: self.currency.clone(),
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            burst: self.burst,
            description: self.description.clone(),
            cost: self.cost,
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation forecast in status proposed.
pub fn new_escalation_forecast(code: &str, name: &str) -> EscalationForecast {
    EscalationForecast {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        currency: "".to_string(),
        timezone: "".to_string(),
        notes: "".to_string(),
        burst: 0,
        description: "".to_string(),
        cost: 0.0,
        ends_at: 0,
        ttl_seconds: 0,
        status: EscalationForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_escalation_forecast("code-433", "name-281");
        assert_eq!(item.status, EscalationForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_escalation_forecast("code-433", "name-281");
        if item.move_to(EscalationForecastStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationForecastStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_escalation_forecast("code-433", "name-281");
        item.status = EscalationForecastStatus::Withdrawn;
        assert!(
            item.move_to(EscalationForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
