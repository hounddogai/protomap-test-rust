use super::*;

use acme_kit::text;

/// Where a binding profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BindingProfileStatus {
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

impl std::fmt::Display for BindingProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding profile as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingProfile {
    pub code: String,
    pub label: String,
    pub description: String,
    pub region: String,
    pub active: bool,
    pub rotation_days: i64,
    pub name: String,
    pub audience: String,
    pub unit: String,
    pub updated_at: i64,
    pub retries: i64,
    pub status: BindingProfileStatus,
}

impl BindingProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingProfileStatus) -> bool {
        match self.status {
            BindingProfileStatus::Planned => {
                match next {
                    BindingProfileStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingProfileStatus::Scheduled => {
                match next {
                    BindingProfileStatus::InProgress | BindingProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingProfileStatus::InProgress => {
                return next == BindingProfileStatus::Completed || next == BindingProfileStatus::Cancelled;
            }
            BindingProfileStatus::Completed => {
                match next {
                    BindingProfileStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BindingProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingProfile {
        BindingProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            region: self.region.clone(),
            active: self.active,
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            audience: self.audience.clone(),
            unit: self.unit.clone(),
            updated_at: self.updated_at,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a binding profile in status planned.
pub fn new_binding_profile(code: &str, label: &str) -> BindingProfile {
    BindingProfile {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        region: "".to_string(),
        active: false,
        rotation_days: 0,
        name: "".to_string(),
        audience: "".to_string(),
        unit: "".to_string(),
        updated_at: 0,
        retries: 0,
        status: BindingProfileStatus::Planned,
    }
}
