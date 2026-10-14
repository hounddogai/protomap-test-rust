use super::*;

use acme_kit::text;

/// Where a regional runbook forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRunbookForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRunbookForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRunbookForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional runbook forecast as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookForecast {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub starts_at: i64,
    pub created_at: i64,
    pub ttl_seconds: i64,
    pub score: f64,
    pub status: RegionalRunbookForecastStatus,
}

impl RegionalRunbookForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRunbookForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRunbookForecastStatus) -> bool {
        match self.status {
            RegionalRunbookForecastStatus::Proposed => {
                match next {
                    RegionalRunbookForecastStatus::Approved | RegionalRunbookForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookForecastStatus::Approved => {
                return next == RegionalRunbookForecastStatus::Rejected
                    || next == RegionalRunbookForecastStatus::Withdrawn;
            }
            RegionalRunbookForecastStatus::Rejected => {
                match next {
                    RegionalRunbookForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalRunbookForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional runbook forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional runbook forecast", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRunbookForecast {
        RegionalRunbookForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            starts_at: self.starts_at,
            created_at: self.created_at,
            ttl_seconds: self.ttl_seconds,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional runbook forecast in status proposed.
pub fn new_regional_runbook_forecast(code: &str, label: &str) -> RegionalRunbookForecast {
    RegionalRunbookForecast {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        starts_at: 0,
        created_at: 0,
        ttl_seconds: 0,
        score: 0.0,
        status: RegionalRunbookForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_runbook_forecast("code-471", "label-913");
        assert_eq!(item.status, RegionalRunbookForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_runbook_forecast("code-471", "label-913");
        if item.move_to(RegionalRunbookForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRunbookForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_runbook_forecast("code-471", "label-913");
        item.status = RegionalRunbookForecastStatus::Withdrawn;
        assert!(
            item.move_to(RegionalRunbookForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
