use super::*;

use acme_kit::text;

/// Where a manual verifier preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualVerifierPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualVerifierPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual verifier preference as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreference {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub fingerprint: String,
    pub quantity: i64,
    pub maximum: f64,
    pub description: String,
    pub score: f64,
    pub status: ManualVerifierPreferenceStatus,
}

impl ManualVerifierPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierPreferenceStatus) -> bool {
        match self.status {
            ManualVerifierPreferenceStatus::Requested => {
                match next {
                    ManualVerifierPreferenceStatus::Granted | ManualVerifierPreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierPreferenceStatus::Granted => {
                return next == ManualVerifierPreferenceStatus::Revoked
                    || next == ManualVerifierPreferenceStatus::Expired;
            }
            ManualVerifierPreferenceStatus::Revoked => {
                return next == ManualVerifierPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualVerifierPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual verifier preference", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierPreference {
        ManualVerifierPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            maximum: self.maximum,
            description: self.description.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual verifier preference in status requested.
pub fn new_manual_verifier_preference(code: &str, title: &str) -> ManualVerifierPreference {
    ManualVerifierPreference {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        fingerprint: "".to_string(),
        quantity: 0,
        maximum: 0.0,
        description: "".to_string(),
        score: 0.0,
        status: ManualVerifierPreferenceStatus::Requested,
    }
}
