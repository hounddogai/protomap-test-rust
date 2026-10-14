use super::*;

use acme_kit::text;

/// Where a zone quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone quota as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneQuota {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub updated_at: i64,
    pub owner_id: String,
    pub issuer: String,
    pub ceiling: f64,
    pub visible: bool,
    pub tenant_id: String,
    pub created_at: i64,
    pub status: ZoneQuotaStatus,
}

impl ZoneQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneQuotaStatus) -> bool {
        match self.status {
            ZoneQuotaStatus::Created => {
                match next {
                    ZoneQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneQuotaStatus::Validated => {
                match next {
                    ZoneQuotaStatus::Published | ZoneQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneQuotaStatus::Published => {
                match next {
                    ZoneQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: ZoneQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.ceiling * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneQuota {
        ZoneQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a zone quota in status created.
pub fn new_zone_quota(code: &str, label: &str) -> ZoneQuota {
    ZoneQuota {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        updated_at: 0,
        owner_id: "".to_string(),
        issuer: "".to_string(),
        ceiling: 0.0,
        visible: false,
        tenant_id: "".to_string(),
        created_at: 0,
        status: ZoneQuotaStatus::Created,
    }
}
