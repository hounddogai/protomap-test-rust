use super::*;

use acme_kit::text;

/// Where a redaction profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RedactionProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RedactionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction profile as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionProfile {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub rate: f64,
    pub duration_minutes: i64,
    pub status: RedactionProfileStatus,
}

impl RedactionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionProfileStatus) -> bool {
        match self.status {
            RedactionProfileStatus::Open => {
                match next {
                    RedactionProfileStatus::Reopened | RedactionProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionProfileStatus::InReview => {
                match next {
                    RedactionProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionProfileStatus::Resolved => {
                match next {
                    RedactionProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionProfileStatus::Reopened => {
                match next {
                    RedactionProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: RedactionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionProfile {
        RedactionProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a redaction profile in status open.
pub fn new_redaction_profile(code: &str, label: &str) -> RedactionProfile {
    RedactionProfile {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        rate: 0.0,
        duration_minutes: 0,
        status: RedactionProfileStatus::Open,
    }
}
