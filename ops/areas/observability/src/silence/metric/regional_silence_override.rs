use super::*;

use acme_kit::text;

/// Where a regional silence override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSilenceOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalSilenceOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalSilenceOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional silence override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceOverride {
    pub code: String,
    pub label: String,
    pub category: String,
    pub updated_at: i64,
    pub expires_at: i64,
    pub unit: String,
    pub quota_units: i64,
    pub archived: bool,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub status: RegionalSilenceOverrideStatus,
}

impl RegionalSilenceOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSilenceOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSilenceOverrideStatus) -> bool {
        match self.status {
            RegionalSilenceOverrideStatus::Requested => {
                return next == RegionalSilenceOverrideStatus::Revoked;
            }
            RegionalSilenceOverrideStatus::Granted => {
                match next {
                    RegionalSilenceOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSilenceOverrideStatus::Revoked => {
                match next {
                    RegionalSilenceOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalSilenceOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional silence override {} cannot move from {} to {}",
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
        format!("{} {}", "regional silence override", self.code)
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
        ((self.updated_at as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSilenceOverride {
        RegionalSilenceOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            unit: self.unit.clone(),
            quota_units: self.quota_units,
            archived: self.archived,
            threshold: self.threshold,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional silence override in status requested.
pub fn new_regional_silence_override(code: &str, label: &str) -> RegionalSilenceOverride {
    RegionalSilenceOverride {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        updated_at: 0,
        expires_at: 0,
        unit: "".to_string(),
        quota_units: 0,
        archived: false,
        threshold: 0.0,
        tags: Vec::new(),
        status: RegionalSilenceOverrideStatus::Requested,
    }
}
