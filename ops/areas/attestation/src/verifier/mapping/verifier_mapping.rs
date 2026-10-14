use super::*;

use acme_kit::text;

/// Where a verifier mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMapping {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub retries: i64,
    pub grace_days: i64,
    pub locale: String,
    pub status: VerifierMappingStatus,
}

impl VerifierMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierMappingStatus) -> bool {
        match self.status {
            VerifierMappingStatus::Created => {
                match next {
                    VerifierMappingStatus::Validated | VerifierMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierMappingStatus::Validated => {
                match next {
                    VerifierMappingStatus::Published | VerifierMappingStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierMappingStatus::Published => {
                match next {
                    VerifierMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: VerifierMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier mapping", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierMapping {
        VerifierMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            retries: self.retries,
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a verifier mapping in status created.
pub fn new_verifier_mapping(code: &str, label: &str) -> VerifierMapping {
    VerifierMapping {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        retries: 0,
        grace_days: 0,
        locale: "".to_string(),
        status: VerifierMappingStatus::Created,
    }
}
