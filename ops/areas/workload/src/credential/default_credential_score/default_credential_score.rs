use super::*;

use acme_kit::text;

/// Where a default credential score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultCredentialScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultCredentialScoreStatus {
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

impl std::fmt::Display for DefaultCredentialScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default credential score as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialScore {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub summary: String,
    pub host: String,
    pub starts_at: i64,
    pub status: DefaultCredentialScoreStatus,
}

impl DefaultCredentialScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultCredentialScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultCredentialScoreStatus) -> bool {
        match self.status {
            DefaultCredentialScoreStatus::Planned => {
                match next {
                    DefaultCredentialScoreStatus::InProgress | DefaultCredentialScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialScoreStatus::Scheduled => {
                match next {
                    DefaultCredentialScoreStatus::InProgress | DefaultCredentialScoreStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialScoreStatus::InProgress => {
                match next {
                    DefaultCredentialScoreStatus::Completed | DefaultCredentialScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialScoreStatus::Completed => {
                match next {
                    DefaultCredentialScoreStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultCredentialScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default credential score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.retries as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultCredentialScore {
        DefaultCredentialScore {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            host: self.host.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default credential score in status planned.
pub fn new_default_credential_score(code: &str, title: &str) -> DefaultCredentialScore {
    DefaultCredentialScore {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        summary: "".to_string(),
        host: "".to_string(),
        starts_at: 0,
        status: DefaultCredentialScoreStatus::Planned,
    }
}
