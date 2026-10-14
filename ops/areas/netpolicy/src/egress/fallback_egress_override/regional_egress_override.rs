use super::*;

use acme_kit::text;

/// Where a regional egress override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEgressOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalEgressOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalEgressOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional egress override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverride {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub host: String,
    pub grace_days: i64,
    pub status: RegionalEgressOverrideStatus,
}

impl RegionalEgressOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEgressOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEgressOverrideStatus) -> bool {
        match self.status {
            RegionalEgressOverrideStatus::Created => {
                match next {
                    RegionalEgressOverrideStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEgressOverrideStatus::Validated => {
                return next == RegionalEgressOverrideStatus::Published;
            }
            RegionalEgressOverrideStatus::Published => {
                match next {
                    RegionalEgressOverrideStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionalEgressOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional egress override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalEgressOverride {
        RegionalEgressOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional egress override in status created.
pub fn new_regional_egress_override(code: &str, title: &str) -> RegionalEgressOverride {
    RegionalEgressOverride {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        host: "".to_string(),
        grace_days: 0,
        status: RegionalEgressOverrideStatus::Created,
    }
}
