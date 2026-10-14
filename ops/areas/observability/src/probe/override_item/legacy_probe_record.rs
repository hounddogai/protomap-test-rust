use super::*;

use acme_kit::text;

/// Where a legacy probe record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyProbeRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyProbeRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe record as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecord {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub locale: String,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub status: LegacyProbeRecordStatus,
}

impl LegacyProbeRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeRecordStatus) -> bool {
        match self.status {
            LegacyProbeRecordStatus::Created => {
                match next {
                    LegacyProbeRecordStatus::Validated | LegacyProbeRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeRecordStatus::Validated => {
                match next {
                    LegacyProbeRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeRecordStatus::Published => {
                return next == LegacyProbeRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyProbeRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> LegacyProbeRecord {
        LegacyProbeRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy probe record in status created.
pub fn new_legacy_probe_record(code: &str, name: &str) -> LegacyProbeRecord {
    LegacyProbeRecord {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        ttl_seconds: 0,
        tags: Vec::new(),
        status: LegacyProbeRecordStatus::Created,
    }
}
