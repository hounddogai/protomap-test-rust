use super::*;

use acme_kit::text;

/// Where a data export quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DataExportQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DataExportQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export quota as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportQuota {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub key_id: String,
    pub sequence: i64,
    pub status: DataExportQuotaStatus,
}

impl DataExportQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportQuotaStatus) -> bool {
        match self.status {
            DataExportQuotaStatus::Created => {
                match next {
                    DataExportQuotaStatus::Validated | DataExportQuotaStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportQuotaStatus::Validated => {
                return next == DataExportQuotaStatus::Published || next == DataExportQuotaStatus::Retired;
            }
            DataExportQuotaStatus::Published => {
                return next == DataExportQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> DataExportQuota {
        DataExportQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a data export quota in status created.
pub fn new_data_export_quota(code: &str, name: &str) -> DataExportQuota {
    DataExportQuota {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        key_id: "".to_string(),
        sequence: 0,
        status: DataExportQuotaStatus::Created,
    }
}
