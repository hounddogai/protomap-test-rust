use super::*;

use acme_kit::text;

/// Where a rolling tenant forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTenantForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingTenantForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingTenantForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling tenant forecast as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecast {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub rotation_days: i64,
    pub ceiling: f64,
    pub created_at: i64,
    pub status: RollingTenantForecastStatus,
}

impl RollingTenantForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTenantForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTenantForecastStatus) -> bool {
        match self.status {
            RollingTenantForecastStatus::New => {
                match next {
                    RollingTenantForecastStatus::Running | RollingTenantForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTenantForecastStatus::Queued => {
                return next == RollingTenantForecastStatus::Running;
            }
            RollingTenantForecastStatus::Running => {
                return next == RollingTenantForecastStatus::Done || next == RollingTenantForecastStatus::Failed;
            }
            RollingTenantForecastStatus::Done => {
                return next == RollingTenantForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingTenantForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling tenant forecast {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.weight * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTenantForecast {
        RollingTenantForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            rotation_days: self.rotation_days,
            ceiling: self.ceiling,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling tenant forecast in status new.
pub fn new_rolling_tenant_forecast(code: &str, title: &str) -> RollingTenantForecast {
    RollingTenantForecast {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        rotation_days: 0,
        ceiling: 0.0,
        created_at: 0,
        status: RollingTenantForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_tenant_forecast("code-566", "title-649");
        assert_eq!(item.status, RollingTenantForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rolling_tenant_forecast("code-566", "title-649");
        if item.move_to(RollingTenantForecastStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTenantForecastStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_tenant_forecast("code-566", "title-649");
        item.status = RollingTenantForecastStatus::Failed;
        assert!(
            item.move_to(RollingTenantForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
