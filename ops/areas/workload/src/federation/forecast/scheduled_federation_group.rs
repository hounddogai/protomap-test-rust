use super::*;

use acme_kit::text;

/// Where a scheduled federation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFederationGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledFederationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledFederationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled federation group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroup {
    pub code: String,
    pub label: String,
    pub description: String,
    pub checksum: String,
    pub port: i64,
    pub summary: String,
    pub title: String,
    pub category: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub created_at: i64,
    pub status: ScheduledFederationGroupStatus,
}

impl ScheduledFederationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFederationGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFederationGroupStatus) -> bool {
        match self.status {
            ScheduledFederationGroupStatus::Proposed => {
                return next == ScheduledFederationGroupStatus::Approved
                    || next == ScheduledFederationGroupStatus::Rejected;
            }
            ScheduledFederationGroupStatus::Approved => {
                match next {
                    ScheduledFederationGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationGroupStatus::Rejected => {
                return next == ScheduledFederationGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFederationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled federation group {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFederationGroup {
        ScheduledFederationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            port: self.port,
            summary: self.summary.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled federation group in status proposed.
pub fn new_scheduled_federation_group(code: &str, label: &str) -> ScheduledFederationGroup {
    ScheduledFederationGroup {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        checksum: "".to_string(),
        port: 0,
        summary: "".to_string(),
        title: "".to_string(),
        category: "".to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        created_at: 0,
        status: ScheduledFederationGroupStatus::Proposed,
    }
}
