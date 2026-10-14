use super::*;

use acme_kit::text;

/// Where a global identity bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIdentityBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalIdentityBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalIdentityBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global identity bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityBundle {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub region: String,
    pub starts_at: i64,
    pub contact_email: String,
    pub expires_at: i64,
    pub version: i64,
    pub unit: String,
    pub status: GlobalIdentityBundleStatus,
}

impl GlobalIdentityBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIdentityBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIdentityBundleStatus) -> bool {
        match self.status {
            GlobalIdentityBundleStatus::Requested => {
                return next == GlobalIdentityBundleStatus::Revoked || next == GlobalIdentityBundleStatus::Expired;
            }
            GlobalIdentityBundleStatus::Granted => {
                return next == GlobalIdentityBundleStatus::Revoked || next == GlobalIdentityBundleStatus::Expired;
            }
            GlobalIdentityBundleStatus::Revoked => {
                match next {
                    GlobalIdentityBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalIdentityBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global identity bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIdentityBundle {
        GlobalIdentityBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            region: self.region.clone(),
            starts_at: self.starts_at,
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            version: self.version,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global identity bundle in status requested.
pub fn new_global_identity_bundle(code: &str, title: &str) -> GlobalIdentityBundle {
    GlobalIdentityBundle {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        region: "".to_string(),
        starts_at: 0,
        contact_email: "".to_string(),
        expires_at: 0,
        version: 0,
        unit: "".to_string(),
        status: GlobalIdentityBundleStatus::Requested,
    }
}
