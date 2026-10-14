use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a region pin snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionPinSnapshotStatus {
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

impl std::fmt::Display for RegionPinSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin snapshot as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSnapshot {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub unit: String,
    pub status: RegionPinSnapshotStatus,
}

impl RegionPinSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinSnapshotStatus) -> bool {
        match self.status {
            RegionPinSnapshotStatus::New => {
                match next {
                    RegionPinSnapshotStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSnapshotStatus::Queued => {
                match next {
                    RegionPinSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSnapshotStatus::Running => {
                return next == RegionPinSnapshotStatus::Failed;
            }
            RegionPinSnapshotStatus::Done => {
                match next {
                    RegionPinSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionPinSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> RegionPinSnapshot {
        RegionPinSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a region pin snapshot in status new.
pub fn new_region_pin_snapshot(code: &str, name: &str) -> RegionPinSnapshot {
    RegionPinSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        status: RegionPinSnapshotStatus::New,
    }
}
