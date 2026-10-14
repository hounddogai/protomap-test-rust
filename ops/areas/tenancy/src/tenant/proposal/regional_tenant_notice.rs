use super::*;

use acme_kit::text;

/// Where a regional tenant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTenantNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalTenantNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalTenantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional tenant notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTenantNotice {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub category: String,
    pub ends_at: i64,
    pub key_id: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub checksum: String,
    pub summary: String,
    pub status: RegionalTenantNoticeStatus,
}

impl RegionalTenantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTenantNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTenantNoticeStatus) -> bool {
        match self.status {
            RegionalTenantNoticeStatus::Created => {
                return next == RegionalTenantNoticeStatus::Validated;
            }
            RegionalTenantNoticeStatus::Validated => {
                match next {
                    RegionalTenantNoticeStatus::Published | RegionalTenantNoticeStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTenantNoticeStatus::Published => {
                return next == RegionalTenantNoticeStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalTenantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional tenant notice {} cannot move from {} to {}",
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

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTenantNotice {
        RegionalTenantNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            ends_at: self.ends_at,
            key_id: self.key_id.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional tenant notice in status created.
pub fn new_regional_tenant_notice(code: &str, label: &str) -> RegionalTenantNotice {
    RegionalTenantNotice {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        category: "".to_string(),
        ends_at: 0,
        key_id: "".to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        status: RegionalTenantNoticeStatus::Created,
    }
}
