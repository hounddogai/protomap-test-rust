use super::*;

use acme_kit::text;

/// Where a default silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSilenceOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultSilenceOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultSilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default silence override as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilenceOverride {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub principal: String,
    pub created_at: i64,
    pub score: f64,
    pub status: DefaultSilenceOverrideStatus,
}

impl DefaultSilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSilenceOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSilenceOverrideStatus) -> bool {
        match self.status {
            DefaultSilenceOverrideStatus::Proposed => {
                return next == DefaultSilenceOverrideStatus::Withdrawn;
            }
            DefaultSilenceOverrideStatus::Approved => {
                return next == DefaultSilenceOverrideStatus::Rejected;
            }
            DefaultSilenceOverrideStatus::Rejected => {
                match next {
                    DefaultSilenceOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultSilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default silence override {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.created_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSilenceOverride {
        DefaultSilenceOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            principal: self.principal.clone(),
            created_at: self.created_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default silence override in status proposed.
pub fn new_default_silence_override(code: &str, title: &str) -> DefaultSilenceOverride {
    DefaultSilenceOverride {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        principal: "".to_string(),
        created_at: 0,
        score: 0.0,
        status: DefaultSilenceOverrideStatus::Proposed,
    }
}
