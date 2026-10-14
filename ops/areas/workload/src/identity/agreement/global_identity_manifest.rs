use super::*;

use acme_kit::text;

/// Where a global identity manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIdentityManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalIdentityManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalIdentityManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global identity manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityManifest {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub country: String,
    pub minimum: f64,
    pub unit: String,
    pub status: GlobalIdentityManifestStatus,
}

impl GlobalIdentityManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIdentityManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIdentityManifestStatus) -> bool {
        match self.status {
            GlobalIdentityManifestStatus::Planned => {
                match next {
                    GlobalIdentityManifestStatus::InProgress | GlobalIdentityManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityManifestStatus::Scheduled => {
                match next {
                    GlobalIdentityManifestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityManifestStatus::InProgress => {
                match next {
                    GlobalIdentityManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityManifestStatus::Completed => {
                match next {
                    GlobalIdentityManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalIdentityManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global identity manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIdentityManifest {
        GlobalIdentityManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            minimum: self.minimum,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global identity manifest in status planned.
pub fn new_global_identity_manifest(code: &str, name: &str) -> GlobalIdentityManifest {
    GlobalIdentityManifest {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        active: false,
        ttl_seconds: 0,
        cipher: "".to_string(),
        country: "".to_string(),
        minimum: 0.0,
        unit: "".to_string(),
        status: GlobalIdentityManifestStatus::Planned,
    }
}
