use super::*;

use acme_kit::text;

/// Where a rolling failover forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFailoverForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingFailoverForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingFailoverForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling failover forecast as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFailoverForecast {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub notes: String,
    pub principal: String,
    pub quantity: i64,
    pub summary: String,
    pub limit: i64,
    pub status: RollingFailoverForecastStatus,
}

impl RollingFailoverForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFailoverForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFailoverForecastStatus) -> bool {
        match self.status {
            RollingFailoverForecastStatus::Open => {
                return next == RollingFailoverForecastStatus::Reopened;
            }
            RollingFailoverForecastStatus::InReview => {
                match next {
                    RollingFailoverForecastStatus::Resolved | RollingFailoverForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFailoverForecastStatus::Resolved => {
                match next {
                    RollingFailoverForecastStatus::Reopened | RollingFailoverForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFailoverForecastStatus::Reopened => {
                match next {
                    RollingFailoverForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingFailoverForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling failover forecast {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFailoverForecast {
        RollingFailoverForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling failover forecast in status open.
pub fn new_rolling_failover_forecast(code: &str, label: &str) -> RollingFailoverForecast {
    RollingFailoverForecast {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        notes: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        summary: "".to_string(),
        limit: 0,
        status: RollingFailoverForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_failover_forecast("code-586", "label-604");
        assert_eq!(item.status, RollingFailoverForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rolling_failover_forecast("code-586", "label-604");
        if item.move_to(RollingFailoverForecastStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFailoverForecastStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_failover_forecast("code-586", "label-604");
        item.status = RollingFailoverForecastStatus::Archived;
        assert!(
            item.move_to(RollingFailoverForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
