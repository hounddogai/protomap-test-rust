use super::*;

use acme_kit::text;

/// Where a legacy alert policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAlertPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyAlertPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyAlertPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy alert policy as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAlertPolicy {
    pub code: String,
    pub label: String,
    pub title: String,
    pub created_at: i64,
    pub burst: i64,
    pub notes: String,
    pub minimum: f64,
    pub starts_at: i64,
    pub capacity: i64,
    pub status: LegacyAlertPolicyStatus,
}

impl LegacyAlertPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAlertPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAlertPolicyStatus) -> bool {
        match self.status {
            LegacyAlertPolicyStatus::Requested => {
                return next == LegacyAlertPolicyStatus::Revoked;
            }
            LegacyAlertPolicyStatus::Granted => {
                match next {
                    LegacyAlertPolicyStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAlertPolicyStatus::Revoked => {
                match next {
                    LegacyAlertPolicyStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyAlertPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy alert policy {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy alert policy", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAlertPolicy {
        LegacyAlertPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            burst: self.burst,
            notes: self.notes.clone(),
            minimum: self.minimum,
            starts_at: self.starts_at,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy alert policy in status requested.
pub fn new_legacy_alert_policy(code: &str, label: &str) -> LegacyAlertPolicy {
    LegacyAlertPolicy {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        created_at: 0,
        burst: 0,
        notes: "".to_string(),
        minimum: 0.0,
        starts_at: 0,
        capacity: 0,
        status: LegacyAlertPolicyStatus::Requested,
    }
}
