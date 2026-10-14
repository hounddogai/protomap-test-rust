use super::*;

use acme_kit::text;

/// Where a lenient grant record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientGrantRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientGrantRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient grant record as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantRecord {
    pub code: String,
    pub name: String,
    pub host: String,
    pub currency: String,
    pub starts_at: i64,
    pub quota_units: i64,
    pub namespace: String,
    pub updated_at: i64,
    pub channel: String,
    pub cipher: String,
    pub archived: bool,
    pub status: LenientGrantRecordStatus,
}

impl LenientGrantRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantRecordStatus) -> bool {
        match self.status {
            LenientGrantRecordStatus::Created => {
                return next == LenientGrantRecordStatus::Validated || next == LenientGrantRecordStatus::Retired;
            }
            LenientGrantRecordStatus::Validated => {
                match next {
                    LenientGrantRecordStatus::Published | LenientGrantRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientGrantRecordStatus::Published => {
                return next == LenientGrantRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientGrantRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quota_units as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantRecord {
        LenientGrantRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            currency: self.currency.clone(),
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient grant record in status created.
pub fn new_lenient_grant_record(code: &str, name: &str) -> LenientGrantRecord {
    LenientGrantRecord {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        currency: "".to_string(),
        starts_at: 0,
        quota_units: 0,
        namespace: "".to_string(),
        updated_at: 0,
        channel: "".to_string(),
        cipher: "".to_string(),
        archived: false,
        status: LenientGrantRecordStatus::Created,
    }
}
