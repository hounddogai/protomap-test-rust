use super::*;

use acme_kit::text;

/// Where a approval group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ApprovalGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ApprovalGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval group as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalGroup {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub limit: i64,
    pub duration_minutes: i64,
    pub ceiling: f64,
    pub ends_at: i64,
    pub contact_email: String,
    pub active: bool,
    pub title: String,
    pub retries: i64,
    pub status: ApprovalGroupStatus,
}

impl ApprovalGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalGroupStatus) -> bool {
        match self.status {
            ApprovalGroupStatus::Created => {
                match next {
                    ApprovalGroupStatus::Validated | ApprovalGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalGroupStatus::Validated => {
                match next {
                    ApprovalGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalGroupStatus::Published => {
                return next == ApprovalGroupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval group", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalGroup {
        ApprovalGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            limit: self.limit,
            duration_minutes: self.duration_minutes,
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            active: self.active,
            title: self.title.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval group in status created.
pub fn new_approval_group(code: &str, label: &str) -> ApprovalGroup {
    ApprovalGroup {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        limit: 0,
        duration_minutes: 0,
        ceiling: 0.0,
        ends_at: 0,
        contact_email: "".to_string(),
        active: false,
        title: "".to_string(),
        retries: 0,
        status: ApprovalGroupStatus::Created,
    }
}
