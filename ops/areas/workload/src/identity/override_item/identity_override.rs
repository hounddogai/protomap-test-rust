use super::*;

use acme_kit::text;

/// Where a identity override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityOverrideStatus {
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

impl std::fmt::Display for IdentityOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverride {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub retries: i64,
    pub visible: bool,
    pub revision: i64,
    pub label: String,
    pub batch_size: i64,
    pub margin: f64,
    pub updated_at: i64,
    pub status: IdentityOverrideStatus,
}

impl IdentityOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityOverrideStatus) -> bool {
        match self.status {
            IdentityOverrideStatus::Planned => {
                match next {
                    IdentityOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityOverrideStatus::Scheduled => {
                match next {
                    IdentityOverrideStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityOverrideStatus::InProgress => {
                return next == IdentityOverrideStatus::Completed || next == IdentityOverrideStatus::Cancelled;
            }
            IdentityOverrideStatus::Completed => {
                return next == IdentityOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.batch_size as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityOverride {
        IdentityOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            retries: self.retries,
            visible: self.visible,
            revision: self.revision,
            label: self.label.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity override in status planned.
pub fn new_identity_override(code: &str, name: &str) -> IdentityOverride {
    IdentityOverride {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        retries: 0,
        visible: false,
        revision: 0,
        label: "".to_string(),
        batch_size: 0,
        margin: 0.0,
        updated_at: 0,
        status: IdentityOverrideStatus::Planned,
    }
}
