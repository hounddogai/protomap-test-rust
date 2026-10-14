use super::*;

use acme_kit::text;

/// Where a silence profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceProfileStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SilenceProfileStatus {
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

impl std::fmt::Display for SilenceProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence profile as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceProfile {
    pub code: String,
    pub label: String,
    pub description: String,
    pub updated_at: i64,
    pub category: String,
    pub expires_at: i64,
    pub issuer: String,
    pub principal: String,
    pub retries: i64,
    pub status: SilenceProfileStatus,
}

impl SilenceProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceProfileStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceProfileStatus) -> bool {
        match self.status {
            SilenceProfileStatus::Planned => {
                return next == SilenceProfileStatus::Cancelled;
            }
            SilenceProfileStatus::Scheduled => {
                match next {
                    SilenceProfileStatus::InProgress | SilenceProfileStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceProfileStatus::InProgress => {
                match next {
                    SilenceProfileStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceProfileStatus::Completed => {
                return next == SilenceProfileStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence profile {} cannot move from {} to {}",
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
        format!("{} {}", "silence profile", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.updated_at as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceProfile {
        SilenceProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            updated_at: self.updated_at,
            category: self.category.clone(),
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a silence profile in status planned.
pub fn new_silence_profile(code: &str, label: &str) -> SilenceProfile {
    SilenceProfile {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        updated_at: 0,
        category: "".to_string(),
        expires_at: 0,
        issuer: "".to_string(),
        principal: "".to_string(),
        retries: 0,
        status: SilenceProfileStatus::Planned,
    }
}
