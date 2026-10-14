use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling zone snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingZoneSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingZoneSnapshotStatus {
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

impl std::fmt::Display for RollingZoneSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling zone snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshot {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub label: String,
    pub visible: bool,
    pub scope: String,
    pub attributes: BTreeMap<String, String>,
    pub owner_id: String,
    pub tenant_id: String,
    pub approved: bool,
    pub status: RollingZoneSnapshotStatus,
}

impl RollingZoneSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingZoneSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingZoneSnapshotStatus) -> bool {
        match self.status {
            RollingZoneSnapshotStatus::Draft => {
                return next == RollingZoneSnapshotStatus::Active || next == RollingZoneSnapshotStatus::Closed;
            }
            RollingZoneSnapshotStatus::Pending => {
                return next == RollingZoneSnapshotStatus::Active || next == RollingZoneSnapshotStatus::Closed;
            }
            RollingZoneSnapshotStatus::Active => {
                return next == RollingZoneSnapshotStatus::Closed;
            }
            RollingZoneSnapshotStatus::Suspended => {
                match next {
                    RollingZoneSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingZoneSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling zone snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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
        self.ceiling
    }

    pub fn copy(&self) -> RollingZoneSnapshot {
        RollingZoneSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            label: self.label.clone(),
            visible: self.visible,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rolling zone snapshot in status draft.
pub fn new_rolling_zone_snapshot(code: &str, name: &str) -> RollingZoneSnapshot {
    RollingZoneSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        label: "".to_string(),
        visible: false,
        scope: "".to_string(),
        attributes: BTreeMap::new(),
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        status: RollingZoneSnapshotStatus::Draft,
    }
}
