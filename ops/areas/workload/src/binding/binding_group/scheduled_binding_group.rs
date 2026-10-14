use super::*;

use acme_kit::text;

/// Where a scheduled binding group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledBindingGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledBindingGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledBindingGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled binding group as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingGroup {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub floor_value: f64,
    pub key_id: String,
    pub label: String,
    pub port: i64,
    pub status: ScheduledBindingGroupStatus,
}

impl ScheduledBindingGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledBindingGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledBindingGroupStatus) -> bool {
        match self.status {
            ScheduledBindingGroupStatus::Requested => {
                match next {
                    ScheduledBindingGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBindingGroupStatus::Granted => {
                match next {
                    ScheduledBindingGroupStatus::Revoked | ScheduledBindingGroupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledBindingGroupStatus::Revoked => {
                return next == ScheduledBindingGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledBindingGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled binding group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledBindingGroup {
        ScheduledBindingGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            key_id: self.key_id.clone(),
            label: self.label.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled binding group in status requested.
pub fn new_scheduled_binding_group(code: &str, title: &str) -> ScheduledBindingGroup {
    ScheduledBindingGroup {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        floor_value: 0.0,
        key_id: "".to_string(),
        label: "".to_string(),
        port: 0,
        status: ScheduledBindingGroupStatus::Requested,
    }
}
