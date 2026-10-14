use super::*;

use acme_kit::text;

/// Where a legacy data export batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDataExportBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyDataExportBatchStatus {
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

impl std::fmt::Display for LegacyDataExportBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy data export batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatch {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub scope: String,
    pub checksum: String,
    pub floor_value: f64,
    pub rate: f64,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub pinned: bool,
    pub channel: String,
    pub status: LegacyDataExportBatchStatus,
}

impl LegacyDataExportBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDataExportBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDataExportBatchStatus) -> bool {
        match self.status {
            LegacyDataExportBatchStatus::Draft => {
                return next == LegacyDataExportBatchStatus::Active;
            }
            LegacyDataExportBatchStatus::Pending => {
                return next == LegacyDataExportBatchStatus::Suspended || next == LegacyDataExportBatchStatus::Closed;
            }
            LegacyDataExportBatchStatus::Active => {
                match next {
                    LegacyDataExportBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDataExportBatchStatus::Suspended => {
                match next {
                    LegacyDataExportBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyDataExportBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy data export batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.rate * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDataExportBatch {
        LegacyDataExportBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            rate: self.rate,
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy data export batch in status draft.
pub fn new_legacy_data_export_batch(code: &str, name: &str) -> LegacyDataExportBatch {
    LegacyDataExportBatch {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        scope: "".to_string(),
        checksum: "".to_string(),
        floor_value: 0.0,
        rate: 0.0,
        expires_at: 0,
        rotation_days: 0,
        pinned: false,
        channel: "".to_string(),
        status: LegacyDataExportBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_data_export_batch("code-372", "name-231");
        assert_eq!(item.status, LegacyDataExportBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_data_export_batch("code-372", "name-231");
        if item.move_to(LegacyDataExportBatchStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDataExportBatchStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_data_export_batch("code-372", "name-231");
        item.status = LegacyDataExportBatchStatus::Closed;
        assert!(
            item.move_to(LegacyDataExportBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
