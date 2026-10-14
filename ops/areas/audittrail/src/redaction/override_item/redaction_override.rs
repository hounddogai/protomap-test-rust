use super::*;

use acme_kit::text;

/// Where a redaction override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RedactionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverride {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub quota_units: i64,
    pub summary: String,
    pub starts_at: i64,
    pub status: RedactionOverrideStatus,
}

impl RedactionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionOverrideStatus) -> bool {
        match self.status {
            RedactionOverrideStatus::Draft => {
                match next {
                    RedactionOverrideStatus::Suspended | RedactionOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionOverrideStatus::Pending => {
                return next == RedactionOverrideStatus::Active || next == RedactionOverrideStatus::Closed;
            }
            RedactionOverrideStatus::Active => {
                match next {
                    RedactionOverrideStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionOverrideStatus::Suspended => {
                return next == RedactionOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction override {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.starts_at as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionOverride {
        RedactionOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction override in status draft.
pub fn new_redaction_override(code: &str, name: &str) -> RedactionOverride {
    RedactionOverride {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        summary: "".to_string(),
        starts_at: 0,
        status: RedactionOverrideStatus::Draft,
    }
}
