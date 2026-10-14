use super::*;

use acme_kit::text;

/// Where a global alert override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAlertOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalAlertOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalAlertOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global alert override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverride {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub namespace: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub capacity: i64,
    pub region: String,
    pub status: GlobalAlertOverrideStatus,
}

impl GlobalAlertOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAlertOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAlertOverrideStatus) -> bool {
        match self.status {
            GlobalAlertOverrideStatus::Proposed => {
                return next == GlobalAlertOverrideStatus::Approved || next == GlobalAlertOverrideStatus::Rejected;
            }
            GlobalAlertOverrideStatus::Approved => {
                return next == GlobalAlertOverrideStatus::Withdrawn;
            }
            GlobalAlertOverrideStatus::Rejected => {
                return next == GlobalAlertOverrideStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalAlertOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global alert override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAlertOverride {
        GlobalAlertOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            namespace: self.namespace.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            capacity: self.capacity,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global alert override in status proposed.
pub fn new_global_alert_override(code: &str, title: &str) -> GlobalAlertOverride {
    GlobalAlertOverride {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        namespace: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        capacity: 0,
        region: "".to_string(),
        status: GlobalAlertOverrideStatus::Proposed,
    }
}
