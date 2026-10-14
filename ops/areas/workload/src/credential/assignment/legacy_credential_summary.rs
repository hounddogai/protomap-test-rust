use super::*;

use acme_kit::text;

/// Where a legacy credential summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyCredentialSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyCredentialSummaryStatus {
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

impl std::fmt::Display for LegacyCredentialSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy credential summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummary {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub revision: i64,
    pub updated_at: i64,
    pub status: LegacyCredentialSummaryStatus,
}

impl LegacyCredentialSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyCredentialSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyCredentialSummaryStatus) -> bool {
        match self.status {
            LegacyCredentialSummaryStatus::Planned => {
                return next == LegacyCredentialSummaryStatus::Scheduled
                    || next == LegacyCredentialSummaryStatus::Completed;
            }
            LegacyCredentialSummaryStatus::Scheduled => {
                match next {
                    LegacyCredentialSummaryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCredentialSummaryStatus::InProgress => {
                match next {
                    LegacyCredentialSummaryStatus::Completed | LegacyCredentialSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCredentialSummaryStatus::Completed => {
                match next {
                    LegacyCredentialSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyCredentialSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy credential summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyCredentialSummary {
        LegacyCredentialSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            revision: self.revision,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy credential summary in status planned.
pub fn new_legacy_credential_summary(code: &str, label: &str) -> LegacyCredentialSummary {
    LegacyCredentialSummary {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        revision: 0,
        updated_at: 0,
        status: LegacyCredentialSummaryStatus::Planned,
    }
}
