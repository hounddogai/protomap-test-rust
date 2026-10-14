use super::*;

use acme_kit::text;

/// Where a federation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FederationBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FederationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation baseline as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationBaseline {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub score: f64,
    pub timezone: String,
    pub attempts: i64,
    pub country: String,
    pub revision: i64,
    pub version: i64,
    pub status: FederationBaselineStatus,
}

impl FederationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationBaselineStatus) -> bool {
        match self.status {
            FederationBaselineStatus::Requested => {
                return next == FederationBaselineStatus::Revoked || next == FederationBaselineStatus::Expired;
            }
            FederationBaselineStatus::Granted => {
                return next == FederationBaselineStatus::Revoked;
            }
            FederationBaselineStatus::Revoked => {
                return next == FederationBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation baseline {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.attempts as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationBaseline {
        FederationBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            score: self.score,
            timezone: self.timezone.clone(),
            attempts: self.attempts,
            country: self.country.clone(),
            revision: self.revision,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a federation baseline in status requested.
pub fn new_federation_baseline(code: &str, label: &str) -> FederationBaseline {
    FederationBaseline {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        score: 0.0,
        timezone: "".to_string(),
        attempts: 0,
        country: "".to_string(),
        revision: 0,
        version: 0,
        status: FederationBaselineStatus::Requested,
    }
}
