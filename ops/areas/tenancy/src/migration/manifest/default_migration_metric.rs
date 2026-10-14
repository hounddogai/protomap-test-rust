use super::*;

use acme_kit::text;

/// Where a default migration metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMigrationMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultMigrationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultMigrationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default migration metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationMetric {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub audience: String,
    pub tenant_id: String,
    pub port: i64,
    pub name: String,
    pub locale: String,
    pub status: DefaultMigrationMetricStatus,
}

impl DefaultMigrationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMigrationMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMigrationMetricStatus) -> bool {
        match self.status {
            DefaultMigrationMetricStatus::Created => {
                match next {
                    DefaultMigrationMetricStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationMetricStatus::Validated => {
                return next == DefaultMigrationMetricStatus::Retired;
            }
            DefaultMigrationMetricStatus::Published => {
                match next {
                    DefaultMigrationMetricStatus::Retired => {
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

    pub fn move_to(&mut self, next: DefaultMigrationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default migration metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default migration metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> DefaultMigrationMetric {
        DefaultMigrationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            name: self.name.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a default migration metric in status created.
pub fn new_default_migration_metric(code: &str, label: &str) -> DefaultMigrationMetric {
    DefaultMigrationMetric {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        audience: "".to_string(),
        tenant_id: "".to_string(),
        port: 0,
        name: "".to_string(),
        locale: "".to_string(),
        status: DefaultMigrationMetricStatus::Created,
    }
}
