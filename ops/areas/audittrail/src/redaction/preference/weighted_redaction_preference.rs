use super::*;

use acme_kit::text;

/// Where a weighted redaction preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRedactionPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedRedactionPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedRedactionPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted redaction preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionPreference {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub archived: bool,
    pub fingerprint: String,
    pub threshold: f64,
    pub expires_at: i64,
    pub created_at: i64,
    pub score: f64,
    pub attempts: i64,
    pub port: i64,
    pub status: WeightedRedactionPreferenceStatus,
}

impl WeightedRedactionPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRedactionPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRedactionPreferenceStatus) -> bool {
        match self.status {
            WeightedRedactionPreferenceStatus::Proposed => {
                match next {
                    WeightedRedactionPreferenceStatus::Rejected | WeightedRedactionPreferenceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRedactionPreferenceStatus::Approved => {
                return next == WeightedRedactionPreferenceStatus::Rejected
                    || next == WeightedRedactionPreferenceStatus::Withdrawn;
            }
            WeightedRedactionPreferenceStatus::Rejected => {
                return next == WeightedRedactionPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRedactionPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted redaction preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted redaction preference", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRedactionPreference {
        WeightedRedactionPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            expires_at: self.expires_at,
            created_at: self.created_at,
            score: self.score,
            attempts: self.attempts,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted redaction preference in status proposed.
pub fn new_weighted_redaction_preference(code: &str, title: &str) -> WeightedRedactionPreference {
    WeightedRedactionPreference {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        archived: false,
        fingerprint: "".to_string(),
        threshold: 0.0,
        expires_at: 0,
        created_at: 0,
        score: 0.0,
        attempts: 0,
        port: 0,
        status: WeightedRedactionPreferenceStatus::Proposed,
    }
}
