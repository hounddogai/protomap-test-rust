use super::*;

use acme_kit::text;

/// Where a verifier preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreference {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub burst: i64,
    pub maximum: f64,
    pub timezone: String,
    pub floor_value: f64,
    pub priority: i64,
    pub amount: f64,
    pub currency: String,
    pub status: VerifierPreferenceStatus,
}

impl VerifierPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierPreferenceStatus) -> bool {
        match self.status {
            VerifierPreferenceStatus::Requested => {
                match next {
                    VerifierPreferenceStatus::Granted | VerifierPreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierPreferenceStatus::Granted => {
                return next == VerifierPreferenceStatus::Revoked || next == VerifierPreferenceStatus::Expired;
            }
            VerifierPreferenceStatus::Revoked => {
                return next == VerifierPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier preference", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.maximum + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierPreference {
        VerifierPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            burst: self.burst,
            maximum: self.maximum,
            timezone: self.timezone.clone(),
            floor_value: self.floor_value,
            priority: self.priority,
            amount: self.amount,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a verifier preference in status requested.
pub fn new_verifier_preference(code: &str, label: &str) -> VerifierPreference {
    VerifierPreference {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        burst: 0,
        maximum: 0.0,
        timezone: "".to_string(),
        floor_value: 0.0,
        priority: 0,
        amount: 0.0,
        currency: "".to_string(),
        status: VerifierPreferenceStatus::Requested,
    }
}
