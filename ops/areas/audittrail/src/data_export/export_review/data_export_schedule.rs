use super::*;

use acme_kit::text;

/// Where a data export schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DataExportScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DataExportScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSchedule {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub owner_id: String,
    pub batch_size: i64,
    pub created_at: i64,
    pub status: DataExportScheduleStatus,
}

impl DataExportSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportScheduleStatus) -> bool {
        match self.status {
            DataExportScheduleStatus::Requested => {
                match next {
                    DataExportScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportScheduleStatus::Granted => {
                match next {
                    DataExportScheduleStatus::Revoked | DataExportScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportScheduleStatus::Revoked => {
                return next == DataExportScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export schedule {} cannot move from {} to {}",
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
        format!("{} {}", "data export schedule", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.batch_size as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportSchedule {
        DataExportSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            batch_size: self.batch_size,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a data export schedule in status requested.
pub fn new_data_export_schedule(code: &str, label: &str) -> DataExportSchedule {
    DataExportSchedule {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        owner_id: "".to_string(),
        batch_size: 0,
        created_at: 0,
        status: DataExportScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_data_export_schedule("code-144", "label-322");
        assert_eq!(item.status, DataExportScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_data_export_schedule("code-144", "label-322");
        if item.move_to(DataExportScheduleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportScheduleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_data_export_schedule("code-144", "label-322");
        item.status = DataExportScheduleStatus::Expired;
        assert!(
            item.move_to(DataExportScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
