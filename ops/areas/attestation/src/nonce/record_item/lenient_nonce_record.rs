use super::*;

use acme_kit::text;

/// Where a lenient nonce record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNonceRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientNonceRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientNonceRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient nonce record as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceRecord {
    pub code: String,
    pub name: String,
    pub country: String,
    pub phone: String,
    pub attempts: i64,
    pub status: LenientNonceRecordStatus,
}

impl LenientNonceRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNonceRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNonceRecordStatus) -> bool {
        match self.status {
            LenientNonceRecordStatus::Created => {
                match next {
                    LenientNonceRecordStatus::Validated | LenientNonceRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientNonceRecordStatus::Validated => {
                match next {
                    LenientNonceRecordStatus::Published | LenientNonceRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientNonceRecordStatus::Published => {
                return next == LenientNonceRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientNonceRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> LenientNonceRecord {
        LenientNonceRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            phone: self.phone.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient nonce record in status created.
pub fn new_lenient_nonce_record(code: &str, name: &str) -> LenientNonceRecord {
    LenientNonceRecord {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        phone: "".to_string(),
        attempts: 0,
        status: LenientNonceRecordStatus::Created,
    }
}
