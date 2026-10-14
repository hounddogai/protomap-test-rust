use super::*;

use acme_kit::text;

/// Where a tiered tenant entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTenantEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredTenantEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredTenantEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered tenant entry as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntry {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub external_ref: String,
    pub region: String,
    pub ceiling: f64,
    pub label: String,
    pub burst: i64,
    pub version: i64,
    pub updated_at: i64,
    pub currency: String,
    pub status: TieredTenantEntryStatus,
}

impl TieredTenantEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTenantEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTenantEntryStatus) -> bool {
        match self.status {
            TieredTenantEntryStatus::Requested => {
                match next {
                    TieredTenantEntryStatus::Granted | TieredTenantEntryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTenantEntryStatus::Granted => {
                return next == TieredTenantEntryStatus::Revoked || next == TieredTenantEntryStatus::Expired;
            }
            TieredTenantEntryStatus::Revoked => {
                return next == TieredTenantEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTenantEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered tenant entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTenantEntry {
        TieredTenantEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            external_ref: self.external_ref.clone(),
            region: self.region.clone(),
            ceiling: self.ceiling,
            label: self.label.clone(),
            burst: self.burst,
            version: self.version,
            updated_at: self.updated_at,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered tenant entry in status requested.
pub fn new_tiered_tenant_entry(code: &str, title: &str) -> TieredTenantEntry {
    TieredTenantEntry {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        external_ref: "".to_string(),
        region: "".to_string(),
        ceiling: 0.0,
        label: "".to_string(),
        burst: 0,
        version: 0,
        updated_at: 0,
        currency: "".to_string(),
        status: TieredTenantEntryStatus::Requested,
    }
}
