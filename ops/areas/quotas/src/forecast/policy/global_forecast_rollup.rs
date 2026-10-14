use super::*;

use acme_kit::text;

/// Where a global forecast rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalForecastRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalForecastRollupStatus {
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

impl std::fmt::Display for GlobalForecastRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global forecast rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastRollup {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub description: String,
    pub ends_at: i64,
    pub port: i64,
    pub revision: i64,
    pub owner_id: String,
    pub status: GlobalForecastRollupStatus,
}

impl GlobalForecastRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalForecastRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalForecastRollupStatus) -> bool {
        match self.status {
            GlobalForecastRollupStatus::Draft => {
                match next {
                    GlobalForecastRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastRollupStatus::Pending => {
                return next == GlobalForecastRollupStatus::Closed;
            }
            GlobalForecastRollupStatus::Active => {
                match next {
                    GlobalForecastRollupStatus::Suspended | GlobalForecastRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalForecastRollupStatus::Suspended => {
                match next {
                    GlobalForecastRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalForecastRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global forecast rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global forecast rollup", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.revision as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalForecastRollup {
        GlobalForecastRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            description: self.description.clone(),
            ends_at: self.ends_at,
            port: self.port,
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global forecast rollup in status draft.
pub fn new_global_forecast_rollup(code: &str, title: &str) -> GlobalForecastRollup {
    GlobalForecastRollup {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        description: "".to_string(),
        ends_at: 0,
        port: 0,
        revision: 0,
        owner_id: "".to_string(),
        status: GlobalForecastRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_forecast_rollup("code-826", "title-298");
        assert_eq!(item.status, GlobalForecastRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_global_forecast_rollup("code-826", "title-298");
        if item.move_to(GlobalForecastRollupStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalForecastRollupStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_forecast_rollup("code-826", "title-298");
        item.status = GlobalForecastRollupStatus::Closed;
        assert!(
            item.move_to(GlobalForecastRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
