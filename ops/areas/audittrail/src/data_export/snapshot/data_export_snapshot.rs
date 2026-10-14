use super::*;

use acme_kit::text;

/// Where a data export snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DataExportSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DataExportSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export snapshot as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshot {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub revision: i64,
    pub budget: f64,
    pub limit: i64,
    pub external_ref: String,
    pub maximum: f64,
    pub ratio: f64,
    pub ends_at: i64,
    pub status: DataExportSnapshotStatus,
}

impl DataExportSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportSnapshotStatus) -> bool {
        match self.status {
            DataExportSnapshotStatus::Requested => {
                match next {
                    DataExportSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportSnapshotStatus::Granted => {
                match next {
                    DataExportSnapshotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportSnapshotStatus::Revoked => {
                return next == DataExportSnapshotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export snapshot", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.ratio * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportSnapshot {
        DataExportSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            revision: self.revision,
            budget: self.budget,
            limit: self.limit,
            external_ref: self.external_ref.clone(),
            maximum: self.maximum,
            ratio: self.ratio,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a data export snapshot in status requested.
pub fn new_data_export_snapshot(code: &str, title: &str) -> DataExportSnapshot {
    DataExportSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        revision: 0,
        budget: 0.0,
        limit: 0,
        external_ref: "".to_string(),
        maximum: 0.0,
        ratio: 0.0,
        ends_at: 0,
        status: DataExportSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_data_export_snapshot("code-452", "title-161");
        assert_eq!(item.status, DataExportSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_data_export_snapshot("code-452", "title-161");
        if item.move_to(DataExportSnapshotStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportSnapshotStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_data_export_snapshot("code-452", "title-161");
        item.status = DataExportSnapshotStatus::Expired;
        assert!(
            item.move_to(DataExportSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
