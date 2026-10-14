use super::*;

use acme_kit::text;

/// Where a legacy measurement notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMeasurementNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyMeasurementNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyMeasurementNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy measurement notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementNotice {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub created_at: i64,
    pub label: String,
    pub sequence: i64,
    pub phone: String,
    pub source: String,
    pub quota_units: i64,
    pub status: LegacyMeasurementNoticeStatus,
}

impl LegacyMeasurementNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMeasurementNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMeasurementNoticeStatus) -> bool {
        match self.status {
            LegacyMeasurementNoticeStatus::Proposed => {
                match next {
                    LegacyMeasurementNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementNoticeStatus::Approved => {
                match next {
                    LegacyMeasurementNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementNoticeStatus::Rejected => {
                return next == LegacyMeasurementNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyMeasurementNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy measurement notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMeasurementNotice {
        LegacyMeasurementNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            created_at: self.created_at,
            label: self.label.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            source: self.source.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy measurement notice in status proposed.
pub fn new_legacy_measurement_notice(code: &str, title: &str) -> LegacyMeasurementNotice {
    LegacyMeasurementNotice {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        created_at: 0,
        label: "".to_string(),
        sequence: 0,
        phone: "".to_string(),
        source: "".to_string(),
        quota_units: 0,
        status: LegacyMeasurementNoticeStatus::Proposed,
    }
}
