use super::*;

use acme_kit::text;

/// Where a legacy data export allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDataExportAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyDataExportAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyDataExportAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy data export allocation as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportAllocation {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub retries: i64,
    pub source: String,
    pub phone: String,
    pub burst: i64,
    pub channel: String,
    pub region: String,
    pub duration_minutes: i64,
    pub sequence: i64,
    pub status: LegacyDataExportAllocationStatus,
}

impl LegacyDataExportAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDataExportAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDataExportAllocationStatus) -> bool {
        match self.status {
            LegacyDataExportAllocationStatus::Proposed => {
                return next == LegacyDataExportAllocationStatus::Approved;
            }
            LegacyDataExportAllocationStatus::Approved => {
                match next {
                    LegacyDataExportAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDataExportAllocationStatus::Rejected => {
                return next == LegacyDataExportAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDataExportAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy data export allocation {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDataExportAllocation {
        LegacyDataExportAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            retries: self.retries,
            source: self.source.clone(),
            phone: self.phone.clone(),
            burst: self.burst,
            channel: self.channel.clone(),
            region: self.region.clone(),
            duration_minutes: self.duration_minutes,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy data export allocation in status proposed.
pub fn new_legacy_data_export_allocation(code: &str, name: &str) -> LegacyDataExportAllocation {
    LegacyDataExportAllocation {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        retries: 0,
        source: "".to_string(),
        phone: "".to_string(),
        burst: 0,
        channel: "".to_string(),
        region: "".to_string(),
        duration_minutes: 0,
        sequence: 0,
        status: LegacyDataExportAllocationStatus::Proposed,
    }
}
