use super::*;

use acme_kit::text;

/// Where a data export cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DataExportCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DataExportCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycle {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub tenant_id: String,
    pub scope: String,
    pub status: DataExportCycleStatus,
}

impl DataExportCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportCycleStatus) -> bool {
        match self.status {
            DataExportCycleStatus::Requested => {
                match next {
                    DataExportCycleStatus::Revoked | DataExportCycleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportCycleStatus::Granted => {
                match next {
                    DataExportCycleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportCycleStatus::Revoked => {
                match next {
                    DataExportCycleStatus::Expired => {
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

    pub fn move_to(&mut self, next: DataExportCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export cycle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> DataExportCycle {
        DataExportCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            tenant_id: self.tenant_id.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a data export cycle in status requested.
pub fn new_data_export_cycle(code: &str, name: &str) -> DataExportCycle {
    DataExportCycle {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        tenant_id: "".to_string(),
        scope: "".to_string(),
        status: DataExportCycleStatus::Requested,
    }
}
