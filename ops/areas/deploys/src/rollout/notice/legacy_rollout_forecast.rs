use super::*;

use acme_kit::text;

/// Where a legacy rollout forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyRolloutForecastStatus {
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

impl std::fmt::Display for LegacyRolloutForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout forecast as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutForecast {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub score: f64,
    pub limit: i64,
    pub status: LegacyRolloutForecastStatus,
}

impl LegacyRolloutForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutForecastStatus) -> bool {
        match self.status {
            LegacyRolloutForecastStatus::Draft => {
                match next {
                    LegacyRolloutForecastStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutForecastStatus::Pending => {
                return next == LegacyRolloutForecastStatus::Active || next == LegacyRolloutForecastStatus::Suspended;
            }
            LegacyRolloutForecastStatus::Active => {
                match next {
                    LegacyRolloutForecastStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutForecastStatus::Suspended => {
                return next == LegacyRolloutForecastStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRolloutForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.limit as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutForecast {
        LegacyRolloutForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            score: self.score,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy rollout forecast in status draft.
pub fn new_legacy_rollout_forecast(code: &str, title: &str) -> LegacyRolloutForecast {
    LegacyRolloutForecast {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        batch_size: 0,
        minimum: 0.0,
        score: 0.0,
        limit: 0,
        status: LegacyRolloutForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_rollout_forecast("code-254", "title-347");
        assert_eq!(item.status, LegacyRolloutForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_rollout_forecast("code-254", "title-347");
        if item.move_to(LegacyRolloutForecastStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRolloutForecastStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_rollout_forecast("code-254", "title-347");
        item.status = LegacyRolloutForecastStatus::Closed;
        assert!(
            item.move_to(LegacyRolloutForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
