use super::*;

use acme_kit::text;

/// Where a strict delegation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDelegationSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictDelegationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictDelegationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict delegation slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationSlot {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub country: String,
    pub margin: f64,
    pub ends_at: i64,
    pub ceiling: f64,
    pub principal: String,
    pub cipher: String,
    pub version: i64,
    pub status: StrictDelegationSlotStatus,
}

impl StrictDelegationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDelegationSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDelegationSlotStatus) -> bool {
        match self.status {
            StrictDelegationSlotStatus::Requested => {
                return next == StrictDelegationSlotStatus::Revoked;
            }
            StrictDelegationSlotStatus::Granted => {
                return next == StrictDelegationSlotStatus::Revoked || next == StrictDelegationSlotStatus::Expired;
            }
            StrictDelegationSlotStatus::Revoked => {
                return next == StrictDelegationSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDelegationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict delegation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDelegationSlot {
        StrictDelegationSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            country: self.country.clone(),
            margin: self.margin,
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict delegation slot in status requested.
pub fn new_strict_delegation_slot(code: &str, title: &str) -> StrictDelegationSlot {
    StrictDelegationSlot {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        country: "".to_string(),
        margin: 0.0,
        ends_at: 0,
        ceiling: 0.0,
        principal: "".to_string(),
        cipher: "".to_string(),
        version: 0,
        status: StrictDelegationSlotStatus::Requested,
    }
}
