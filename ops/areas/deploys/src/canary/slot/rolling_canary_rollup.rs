use super::*;

use acme_kit::text;

/// Where a rolling canary rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingCanaryRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingCanaryRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingCanaryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling canary rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryRollup {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub timezone: String,
    pub window_days: i64,
    pub archived: bool,
    pub status: RollingCanaryRollupStatus,
}

impl RollingCanaryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingCanaryRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingCanaryRollupStatus) -> bool {
        match self.status {
            RollingCanaryRollupStatus::Created => {
                match next {
                    RollingCanaryRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingCanaryRollupStatus::Validated => {
                match next {
                    RollingCanaryRollupStatus::Published | RollingCanaryRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingCanaryRollupStatus::Published => {
                return next == RollingCanaryRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingCanaryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling canary rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling canary rollup", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.window_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingCanaryRollup {
        RollingCanaryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling canary rollup in status created.
pub fn new_rolling_canary_rollup(code: &str, title: &str) -> RollingCanaryRollup {
    RollingCanaryRollup {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        label: "".to_string(),
        ttl_seconds: 0,
        timezone: "".to_string(),
        window_days: 0,
        archived: false,
        status: RollingCanaryRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_canary_rollup("code-88", "title-485");
        assert_eq!(item.status, RollingCanaryRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_rolling_canary_rollup("code-88", "title-485");
        if item.move_to(RollingCanaryRollupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingCanaryRollupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_canary_rollup("code-88", "title-485");
        item.status = RollingCanaryRollupStatus::Retired;
        assert!(
            item.move_to(RollingCanaryRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
