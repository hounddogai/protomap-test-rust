use super::*;

use acme_kit::text;

/// Where a manual data export checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualDataExportCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualDataExportCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual data export checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpoint {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub owner_id: String,
    pub timezone: String,
    pub status: ManualDataExportCheckpointStatus,
}

impl ManualDataExportCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportCheckpointStatus) -> bool {
        match self.status {
            ManualDataExportCheckpointStatus::Requested => {
                return next == ManualDataExportCheckpointStatus::Granted
                    || next == ManualDataExportCheckpointStatus::Revoked;
            }
            ManualDataExportCheckpointStatus::Granted => {
                return next == ManualDataExportCheckpointStatus::Expired;
            }
            ManualDataExportCheckpointStatus::Revoked => {
                match next {
                    ManualDataExportCheckpointStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualDataExportCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> ManualDataExportCheckpoint {
        ManualDataExportCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            owner_id: self.owner_id.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual data export checkpoint in status requested.
pub fn new_manual_data_export_checkpoint(code: &str, name: &str) -> ManualDataExportCheckpoint {
    ManualDataExportCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        owner_id: "".to_string(),
        timezone: "".to_string(),
        status: ManualDataExportCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_data_export_checkpoint("code-596", "name-536");
        assert_eq!(item.status, ManualDataExportCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_data_export_checkpoint("code-596", "name-536");
        if item.move_to(ManualDataExportCheckpointStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDataExportCheckpointStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_data_export_checkpoint("code-596", "name-536");
        item.status = ManualDataExportCheckpointStatus::Expired;
        assert!(
            item.move_to(ManualDataExportCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
