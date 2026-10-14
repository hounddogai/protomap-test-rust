use super::*;

use acme_kit::text;

/// Where a rolling migration metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingMigrationMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingMigrationMetricStatus {
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

impl std::fmt::Display for RollingMigrationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling migration metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationMetric {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub limit: i64,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub locale: String,
    pub external_ref: String,
    pub active: bool,
    pub status: RollingMigrationMetricStatus,
}

impl RollingMigrationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingMigrationMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingMigrationMetricStatus) -> bool {
        match self.status {
            RollingMigrationMetricStatus::Draft => {
                match next {
                    RollingMigrationMetricStatus::Suspended | RollingMigrationMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingMigrationMetricStatus::Pending => {
                match next {
                    RollingMigrationMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingMigrationMetricStatus::Active => {
                match next {
                    RollingMigrationMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingMigrationMetricStatus::Suspended => {
                return next == RollingMigrationMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingMigrationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling migration metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.limit as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingMigrationMetric {
        RollingMigrationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            limit: self.limit,
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling migration metric in status draft.
pub fn new_rolling_migration_metric(code: &str, label: &str) -> RollingMigrationMetric {
    RollingMigrationMetric {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        limit: 0,
        pinned: false,
        ttl_seconds: 0,
        floor_value: 0.0,
        locale: "".to_string(),
        external_ref: "".to_string(),
        active: false,
        status: RollingMigrationMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_migration_metric("code-169", "label-400");
        assert_eq!(item.status, RollingMigrationMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_migration_metric("code-169", "label-400");
        if item.move_to(RollingMigrationMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingMigrationMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_migration_metric("code-169", "label-400");
        item.status = RollingMigrationMetricStatus::Closed;
        assert!(
            item.move_to(RollingMigrationMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
