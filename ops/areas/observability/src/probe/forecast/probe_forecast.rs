use super::*;

use acme_kit::text;

/// Where a probe forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeForecastStatus {
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

impl std::fmt::Display for ProbeForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe forecast as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeForecast {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub priority: i64,
    pub timezone: String,
    pub summary: String,
    pub duration_minutes: i64,
    pub status: ProbeForecastStatus,
}

impl ProbeForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeForecastStatus) -> bool {
        match self.status {
            ProbeForecastStatus::Draft => {
                match next {
                    ProbeForecastStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeForecastStatus::Pending => {
                match next {
                    ProbeForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeForecastStatus::Active => {
                return next == ProbeForecastStatus::Suspended || next == ProbeForecastStatus::Closed;
            }
            ProbeForecastStatus::Suspended => {
                return next == ProbeForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "probe forecast", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeForecast {
        ProbeForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            priority: self.priority,
            timezone: self.timezone.clone(),
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe forecast in status draft.
pub fn new_probe_forecast(code: &str, label: &str) -> ProbeForecast {
    ProbeForecast {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        priority: 0,
        timezone: "".to_string(),
        summary: "".to_string(),
        duration_minutes: 0,
        status: ProbeForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_forecast("code-752", "label-94");
        assert_eq!(item.status, ProbeForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_probe_forecast("code-752", "label-94");
        if item.move_to(ProbeForecastStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeForecastStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_forecast("code-752", "label-94");
        item.status = ProbeForecastStatus::Closed;
        assert!(
            item.move_to(ProbeForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
