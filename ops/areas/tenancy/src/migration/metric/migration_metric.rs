use super::*;

use acme_kit::text;

/// Where a migration metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MigrationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for MigrationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationMetric {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub rotation_days: i64,
    pub priority: i64,
    pub region: String,
    pub minimum: f64,
    pub currency: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub approved: bool,
    pub status: MigrationMetricStatus,
}

impl MigrationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationMetricStatus) -> bool {
        match self.status {
            MigrationMetricStatus::Idle => {
                return next == MigrationMetricStatus::Warming || next == MigrationMetricStatus::Ready;
            }
            MigrationMetricStatus::Warming => {
                match next {
                    MigrationMetricStatus::Ready | MigrationMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationMetricStatus::Ready => {
                match next {
                    MigrationMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationMetricStatus::Draining => {
                match next {
                    MigrationMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MigrationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "migration metric", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.minimum - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationMetric {
        MigrationMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            priority: self.priority,
            region: self.region.clone(),
            minimum: self.minimum,
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a migration metric in status idle.
pub fn new_migration_metric(code: &str, name: &str) -> MigrationMetric {
    MigrationMetric {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        priority: 0,
        region: "".to_string(),
        minimum: 0.0,
        currency: "".to_string(),
        duration_minutes: 0,
        updated_at: 0,
        approved: false,
        status: MigrationMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_migration_metric("code-761", "name-574");
        assert_eq!(item.status, MigrationMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_migration_metric("code-761", "name-574");
        if item.move_to(MigrationMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_migration_metric("code-761", "name-574");
        item.status = MigrationMetricStatus::Stopped;
        assert!(
            item.move_to(MigrationMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
