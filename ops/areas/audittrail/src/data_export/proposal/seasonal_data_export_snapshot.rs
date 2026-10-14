use super::*;

use acme_kit::text;

/// Where a seasonal data export snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDataExportSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalDataExportSnapshotStatus {
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

impl std::fmt::Display for SeasonalDataExportSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal data export snapshot as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportSnapshot {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub attempts: i64,
    pub timezone: String,
    pub key_id: String,
    pub expires_at: i64,
    pub source: String,
    pub window_days: i64,
    pub status: SeasonalDataExportSnapshotStatus,
}

impl SeasonalDataExportSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDataExportSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDataExportSnapshotStatus) -> bool {
        match self.status {
            SeasonalDataExportSnapshotStatus::Draft => {
                match next {
                    SeasonalDataExportSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDataExportSnapshotStatus::Pending => {
                match next {
                    SeasonalDataExportSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDataExportSnapshotStatus::Active => {
                return next == SeasonalDataExportSnapshotStatus::Closed;
            }
            SeasonalDataExportSnapshotStatus::Suspended => {
                return next == SeasonalDataExportSnapshotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalDataExportSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal data export snapshot {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.expires_at as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDataExportSnapshot {
        SeasonalDataExportSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            key_id: self.key_id.clone(),
            expires_at: self.expires_at,
            source: self.source.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal data export snapshot in status draft.
pub fn new_seasonal_data_export_snapshot(code: &str, title: &str) -> SeasonalDataExportSnapshot {
    SeasonalDataExportSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        attempts: 0,
        timezone: "".to_string(),
        key_id: "".to_string(),
        expires_at: 0,
        source: "".to_string(),
        window_days: 0,
        status: SeasonalDataExportSnapshotStatus::Draft,
    }
}
