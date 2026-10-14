use super::*;

use acme_kit::text;

/// Where a scheduled delegation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDelegationProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledDelegationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledDelegationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled delegation profile as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDelegationProfile {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub port: i64,
    pub phone: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub status: ScheduledDelegationProfileStatus,
}

impl ScheduledDelegationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDelegationProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDelegationProfileStatus) -> bool {
        match self.status {
            ScheduledDelegationProfileStatus::Requested => {
                match next {
                    ScheduledDelegationProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDelegationProfileStatus::Granted => {
                return next == ScheduledDelegationProfileStatus::Revoked;
            }
            ScheduledDelegationProfileStatus::Revoked => {
                return next == ScheduledDelegationProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDelegationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled delegation profile {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDelegationProfile {
        ScheduledDelegationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            port: self.port,
            phone: self.phone.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled delegation profile in status requested.
pub fn new_scheduled_delegation_profile(code: &str, title: &str) -> ScheduledDelegationProfile {
    ScheduledDelegationProfile {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        port: 0,
        phone: "".to_string(),
        tags: Vec::new(),
        created_at: 0,
        status: ScheduledDelegationProfileStatus::Requested,
    }
}
