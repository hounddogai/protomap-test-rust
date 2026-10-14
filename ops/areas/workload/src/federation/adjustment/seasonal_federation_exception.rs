use super::*;

use acme_kit::text;

/// Where a seasonal federation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFederationExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalFederationExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for SeasonalFederationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal federation exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationException {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub country: String,
    pub limit: i64,
    pub notes: String,
    pub status: SeasonalFederationExceptionStatus,
}

impl SeasonalFederationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFederationExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFederationExceptionStatus) -> bool {
        match self.status {
            SeasonalFederationExceptionStatus::New => {
                match next {
                    SeasonalFederationExceptionStatus::Queued | SeasonalFederationExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationExceptionStatus::Queued => {
                match next {
                    SeasonalFederationExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationExceptionStatus::Running => {
                match next {
                    SeasonalFederationExceptionStatus::Done | SeasonalFederationExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationExceptionStatus::Done => {
                match next {
                    SeasonalFederationExceptionStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalFederationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal federation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.maximum * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFederationException {
        SeasonalFederationException {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            country: self.country.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal federation exception in status new.
pub fn new_seasonal_federation_exception(code: &str, label: &str) -> SeasonalFederationException {
    SeasonalFederationException {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        country: "".to_string(),
        limit: 0,
        notes: "".to_string(),
        status: SeasonalFederationExceptionStatus::New,
    }
}
