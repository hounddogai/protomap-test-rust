use super::*;

use acme_kit::text;

/// Where a global data export policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDataExportPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalDataExportPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalDataExportPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global data export policy as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicy {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub country: String,
    pub timezone: String,
    pub notes: String,
    pub version: i64,
    pub cost: f64,
    pub segment: String,
    pub status: GlobalDataExportPolicyStatus,
}

impl GlobalDataExportPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDataExportPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDataExportPolicyStatus) -> bool {
        match self.status {
            GlobalDataExportPolicyStatus::Requested => {
                return next == GlobalDataExportPolicyStatus::Expired;
            }
            GlobalDataExportPolicyStatus::Granted => {
                match next {
                    GlobalDataExportPolicyStatus::Revoked | GlobalDataExportPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDataExportPolicyStatus::Revoked => {
                return next == GlobalDataExportPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalDataExportPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global data export policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global data export policy", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDataExportPolicy {
        GlobalDataExportPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            version: self.version,
            cost: self.cost,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global data export policy in status requested.
pub fn new_global_data_export_policy(code: &str, name: &str) -> GlobalDataExportPolicy {
    GlobalDataExportPolicy {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        country: "".to_string(),
        timezone: "".to_string(),
        notes: "".to_string(),
        version: 0,
        cost: 0.0,
        segment: "".to_string(),
        status: GlobalDataExportPolicyStatus::Requested,
    }
}
