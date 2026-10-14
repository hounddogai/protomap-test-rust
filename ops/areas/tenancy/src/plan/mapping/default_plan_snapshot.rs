use super::*;

use acme_kit::text;

/// Where a default plan snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPlanSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultPlanSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultPlanSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default plan snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshot {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub cipher: String,
    pub ceiling: f64,
    pub owner_id: String,
    pub updated_at: i64,
    pub channel: String,
    pub status: DefaultPlanSnapshotStatus,
}

impl DefaultPlanSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPlanSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPlanSnapshotStatus) -> bool {
        match self.status {
            DefaultPlanSnapshotStatus::Created => {
                return next == DefaultPlanSnapshotStatus::Validated || next == DefaultPlanSnapshotStatus::Retired;
            }
            DefaultPlanSnapshotStatus::Validated => {
                return next == DefaultPlanSnapshotStatus::Published || next == DefaultPlanSnapshotStatus::Retired;
            }
            DefaultPlanSnapshotStatus::Published => {
                match next {
                    DefaultPlanSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: DefaultPlanSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default plan snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default plan snapshot", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ceiling + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPlanSnapshot {
        DefaultPlanSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            ceiling: self.ceiling,
            owner_id: self.owner_id.clone(),
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default plan snapshot in status created.
pub fn new_default_plan_snapshot(code: &str, name: &str) -> DefaultPlanSnapshot {
    DefaultPlanSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        cipher: "".to_string(),
        ceiling: 0.0,
        owner_id: "".to_string(),
        updated_at: 0,
        channel: "".to_string(),
        status: DefaultPlanSnapshotStatus::Created,
    }
}
