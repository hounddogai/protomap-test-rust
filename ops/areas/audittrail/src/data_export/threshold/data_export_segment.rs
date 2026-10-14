use super::*;

use acme_kit::text;

/// Where a data export segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DataExportSegmentStatus {
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

impl std::fmt::Display for DataExportSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSegment {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub tenant_id: String,
    pub category: String,
    pub capacity: i64,
    pub ends_at: i64,
    pub grace_days: i64,
    pub cost: f64,
    pub issuer: String,
    pub cipher: String,
    pub status: DataExportSegmentStatus,
}

impl DataExportSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportSegmentStatus) -> bool {
        match self.status {
            DataExportSegmentStatus::Draft => {
                return next == DataExportSegmentStatus::Suspended || next == DataExportSegmentStatus::Closed;
            }
            DataExportSegmentStatus::Pending => {
                return next == DataExportSegmentStatus::Closed;
            }
            DataExportSegmentStatus::Active => {
                match next {
                    DataExportSegmentStatus::Suspended | DataExportSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportSegmentStatus::Suspended => {
                match next {
                    DataExportSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: DataExportSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportSegment {
        DataExportSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            category: self.category.clone(),
            capacity: self.capacity,
            ends_at: self.ends_at,
            grace_days: self.grace_days,
            cost: self.cost,
            issuer: self.issuer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a data export segment in status draft.
pub fn new_data_export_segment(code: &str, label: &str) -> DataExportSegment {
    DataExportSegment {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        category: "".to_string(),
        capacity: 0,
        ends_at: 0,
        grace_days: 0,
        cost: 0.0,
        issuer: "".to_string(),
        cipher: "".to_string(),
        status: DataExportSegmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_data_export_segment("code-327", "label-465");
        assert_eq!(item.status, DataExportSegmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_data_export_segment("code-327", "label-465");
        if item.move_to(DataExportSegmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportSegmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_data_export_segment("code-327", "label-465");
        item.status = DataExportSegmentStatus::Closed;
        assert!(
            item.move_to(DataExportSegmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
