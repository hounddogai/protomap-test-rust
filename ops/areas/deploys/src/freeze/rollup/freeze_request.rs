use super::*;

use acme_kit::text;

/// Where a freeze request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FreezeRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FreezeRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRequest {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub region: String,
    pub tenant_id: String,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub category: String,
    pub external_ref: String,
    pub status: FreezeRequestStatus,
}

impl FreezeRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeRequestStatus) -> bool {
        match self.status {
            FreezeRequestStatus::Requested => {
                return next == FreezeRequestStatus::Expired;
            }
            FreezeRequestStatus::Granted => {
                match next {
                    FreezeRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeRequestStatus::Revoked => {
                match next {
                    FreezeRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: FreezeRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze request {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeRequest {
        FreezeRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a freeze request in status requested.
pub fn new_freeze_request(code: &str, label: &str) -> FreezeRequest {
    FreezeRequest {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        region: "".to_string(),
        tenant_id: "".to_string(),
        starts_at: 0,
        duration_minutes: 0,
        category: "".to_string(),
        external_ref: "".to_string(),
        status: FreezeRequestStatus::Requested,
    }
}
