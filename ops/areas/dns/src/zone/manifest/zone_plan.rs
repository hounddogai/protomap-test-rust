use super::*;

use acme_kit::text;

/// Where a zone plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZonePlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ZonePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ZonePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlan {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub created_at: i64,
    pub country: String,
    pub approved: bool,
    pub checksum: String,
    pub cost: f64,
    pub contact_email: String,
    pub description: String,
    pub status: ZonePlanStatus,
}

impl ZonePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZonePlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZonePlanStatus) -> bool {
        match self.status {
            ZonePlanStatus::Requested => {
                return next == ZonePlanStatus::Granted;
            }
            ZonePlanStatus::Granted => {
                match next {
                    ZonePlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZonePlanStatus::Revoked => {
                return next == ZonePlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZonePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.created_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZonePlan {
        ZonePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            created_at: self.created_at,
            country: self.country.clone(),
            approved: self.approved,
            checksum: self.checksum.clone(),
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a zone plan in status requested.
pub fn new_zone_plan(code: &str, name: &str) -> ZonePlan {
    ZonePlan {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        created_at: 0,
        country: "".to_string(),
        approved: false,
        checksum: "".to_string(),
        cost: 0.0,
        contact_email: "".to_string(),
        description: "".to_string(),
        status: ZonePlanStatus::Requested,
    }
}
