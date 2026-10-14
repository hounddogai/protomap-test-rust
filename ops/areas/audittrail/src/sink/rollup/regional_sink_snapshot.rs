use super::*;

use acme_kit::text;

/// Where a regional sink snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSinkSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalSinkSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalSinkSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional sink snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSinkSnapshot {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub minimum: f64,
    pub summary: String,
    pub burst: i64,
    pub port: i64,
    pub locale: String,
    pub retries: i64,
    pub budget: f64,
    pub margin: f64,
    pub status: RegionalSinkSnapshotStatus,
}

impl RegionalSinkSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSinkSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSinkSnapshotStatus) -> bool {
        match self.status {
            RegionalSinkSnapshotStatus::Created => {
                match next {
                    RegionalSinkSnapshotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSinkSnapshotStatus::Validated => {
                return next == RegionalSinkSnapshotStatus::Published || next == RegionalSinkSnapshotStatus::Retired;
            }
            RegionalSinkSnapshotStatus::Published => {
                return next == RegionalSinkSnapshotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSinkSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional sink snapshot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional sink snapshot", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.budget - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSinkSnapshot {
        RegionalSinkSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            minimum: self.minimum,
            summary: self.summary.clone(),
            burst: self.burst,
            port: self.port,
            locale: self.locale.clone(),
            retries: self.retries,
            budget: self.budget,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional sink snapshot in status created.
pub fn new_regional_sink_snapshot(code: &str, label: &str) -> RegionalSinkSnapshot {
    RegionalSinkSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        minimum: 0.0,
        summary: "".to_string(),
        burst: 0,
        port: 0,
        locale: "".to_string(),
        retries: 0,
        budget: 0.0,
        margin: 0.0,
        status: RegionalSinkSnapshotStatus::Created,
    }
}
