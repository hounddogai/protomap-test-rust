use super::*;

use acme_kit::text;

/// Where a legacy nonce forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyNonceForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyNonceForecastStatus {
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

impl std::fmt::Display for LegacyNonceForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy nonce forecast as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecast {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub weight: f64,
    pub external_ref: String,
    pub grace_days: i64,
    pub namespace: String,
    pub description: String,
    pub status: LegacyNonceForecastStatus,
}

impl LegacyNonceForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyNonceForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyNonceForecastStatus) -> bool {
        match self.status {
            LegacyNonceForecastStatus::Planned => {
                return next == LegacyNonceForecastStatus::InProgress || next == LegacyNonceForecastStatus::Cancelled;
            }
            LegacyNonceForecastStatus::Scheduled => {
                return next == LegacyNonceForecastStatus::InProgress || next == LegacyNonceForecastStatus::Cancelled;
            }
            LegacyNonceForecastStatus::InProgress => {
                match next {
                    LegacyNonceForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyNonceForecastStatus::Completed => {
                return next == LegacyNonceForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyNonceForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy nonce forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.grace_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyNonceForecast {
        LegacyNonceForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            weight: self.weight,
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy nonce forecast in status planned.
pub fn new_legacy_nonce_forecast(code: &str, title: &str) -> LegacyNonceForecast {
    LegacyNonceForecast {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        weight: 0.0,
        external_ref: "".to_string(),
        grace_days: 0,
        namespace: "".to_string(),
        description: "".to_string(),
        status: LegacyNonceForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_nonce_forecast("code-803", "title-389");
        assert_eq!(item.status, LegacyNonceForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_legacy_nonce_forecast("code-803", "title-389");
        if item.move_to(LegacyNonceForecastStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyNonceForecastStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_nonce_forecast("code-803", "title-389");
        item.status = LegacyNonceForecastStatus::Cancelled;
        assert!(
            item.move_to(LegacyNonceForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
