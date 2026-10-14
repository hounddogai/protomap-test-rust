use super::*;

use acme_kit::text;

/// Where a regional peering quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPeeringQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalPeeringQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalPeeringQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional peering quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringQuota {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub updated_at: i64,
    pub budget: f64,
    pub key_id: String,
    pub segment: String,
    pub channel: String,
    pub tenant_id: String,
    pub status: RegionalPeeringQuotaStatus,
}

impl RegionalPeeringQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPeeringQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPeeringQuotaStatus) -> bool {
        match self.status {
            RegionalPeeringQuotaStatus::Idle => {
                return next == RegionalPeeringQuotaStatus::Warming || next == RegionalPeeringQuotaStatus::Stopped;
            }
            RegionalPeeringQuotaStatus::Warming => {
                return next == RegionalPeeringQuotaStatus::Draining;
            }
            RegionalPeeringQuotaStatus::Ready => {
                return next == RegionalPeeringQuotaStatus::Stopped;
            }
            RegionalPeeringQuotaStatus::Draining => {
                return next == RegionalPeeringQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPeeringQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional peering quota {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPeeringQuota {
        RegionalPeeringQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            updated_at: self.updated_at,
            budget: self.budget,
            key_id: self.key_id.clone(),
            segment: self.segment.clone(),
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional peering quota in status idle.
pub fn new_regional_peering_quota(code: &str, label: &str) -> RegionalPeeringQuota {
    RegionalPeeringQuota {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        updated_at: 0,
        budget: 0.0,
        key_id: "".to_string(),
        segment: "".to_string(),
        channel: "".to_string(),
        tenant_id: "".to_string(),
        status: RegionalPeeringQuotaStatus::Idle,
    }
}
