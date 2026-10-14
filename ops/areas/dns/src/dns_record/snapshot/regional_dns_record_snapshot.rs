use super::*;

use acme_kit::text;

/// Where a regional dns record snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDnsRecordSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalDnsRecordSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalDnsRecordSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional dns record snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshot {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub grace_days: i64,
    pub tenant_id: String,
    pub timezone: String,
    pub status: RegionalDnsRecordSnapshotStatus,
}

impl RegionalDnsRecordSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDnsRecordSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDnsRecordSnapshotStatus) -> bool {
        match self.status {
            RegionalDnsRecordSnapshotStatus::Created => {
                return next == RegionalDnsRecordSnapshotStatus::Validated;
            }
            RegionalDnsRecordSnapshotStatus::Validated => {
                match next {
                    RegionalDnsRecordSnapshotStatus::Published | RegionalDnsRecordSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDnsRecordSnapshotStatus::Published => {
                return next == RegionalDnsRecordSnapshotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDnsRecordSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional dns record snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional dns record snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> RegionalDnsRecordSnapshot {
        RegionalDnsRecordSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            grace_days: self.grace_days,
            tenant_id: self.tenant_id.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional dns record snapshot in status created.
pub fn new_regional_dns_record_snapshot(code: &str, name: &str) -> RegionalDnsRecordSnapshot {
    RegionalDnsRecordSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        grace_days: 0,
        tenant_id: "".to_string(),
        timezone: "".to_string(),
        status: RegionalDnsRecordSnapshotStatus::Created,
    }
}
