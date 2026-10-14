use super::*;

use acme_kit::text;

/// Where a capped migration forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedMigrationForecastStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedMigrationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedMigrationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped migration forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecast {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub region: String,
    pub ends_at: i64,
    pub threshold: f64,
    pub principal: String,
    pub ttl_seconds: i64,
    pub status: CappedMigrationForecastStatus,
}

impl CappedMigrationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedMigrationForecastStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedMigrationForecastStatus) -> bool {
        match self.status {
            CappedMigrationForecastStatus::Created => {
                return next == CappedMigrationForecastStatus::Published;
            }
            CappedMigrationForecastStatus::Validated => {
                return next == CappedMigrationForecastStatus::Retired;
            }
            CappedMigrationForecastStatus::Published => {
                return next == CappedMigrationForecastStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedMigrationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped migration forecast {} cannot move from {} to {}",
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
        format!("{} {}", "capped migration forecast", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedMigrationForecast {
        CappedMigrationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            ends_at: self.ends_at,
            threshold: self.threshold,
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped migration forecast in status created.
pub fn new_capped_migration_forecast(code: &str, title: &str) -> CappedMigrationForecast {
    CappedMigrationForecast {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        region: "".to_string(),
        ends_at: 0,
        threshold: 0.0,
        principal: "".to_string(),
        ttl_seconds: 0,
        status: CappedMigrationForecastStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_migration_forecast("code-140", "title-913");
        assert_eq!(item.status, CappedMigrationForecastStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_migration_forecast("code-140", "title-913");
        if item.move_to(CappedMigrationForecastStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedMigrationForecastStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_migration_forecast("code-140", "title-913");
        item.status = CappedMigrationForecastStatus::Retired;
        assert!(
            item.move_to(CappedMigrationForecastStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
