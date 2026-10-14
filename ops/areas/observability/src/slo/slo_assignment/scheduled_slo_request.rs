use super::*;

use acme_kit::text;

/// Where a scheduled slo request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSloRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledSloRequestStatus {
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

impl std::fmt::Display for ScheduledSloRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled slo request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloRequest {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub unit: String,
    pub cluster: String,
    pub expires_at: i64,
    pub checksum: String,
    pub contact_email: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub status: ScheduledSloRequestStatus,
}

impl ScheduledSloRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSloRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSloRequestStatus) -> bool {
        match self.status {
            ScheduledSloRequestStatus::Draft => {
                return next == ScheduledSloRequestStatus::Active;
            }
            ScheduledSloRequestStatus::Pending => {
                match next {
                    ScheduledSloRequestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSloRequestStatus::Active => {
                match next {
                    ScheduledSloRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSloRequestStatus::Suspended => {
                match next {
                    ScheduledSloRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledSloRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled slo request {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSloRequest {
        ScheduledSloRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            expires_at: self.expires_at,
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled slo request in status draft.
pub fn new_scheduled_slo_request(code: &str, name: &str) -> ScheduledSloRequest {
    ScheduledSloRequest {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        unit: "".to_string(),
        cluster: "".to_string(),
        expires_at: 0,
        checksum: "".to_string(),
        contact_email: "".to_string(),
        active: false,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        status: ScheduledSloRequestStatus::Draft,
    }
}
