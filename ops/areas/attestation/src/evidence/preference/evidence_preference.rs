use super::*;

use acme_kit::text;

/// Where a evidence preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidencePreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EvidencePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EvidencePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePreference {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub tenant_id: String,
    pub port: i64,
    pub status: EvidencePreferenceStatus,
}

impl EvidencePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidencePreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidencePreferenceStatus) -> bool {
        match self.status {
            EvidencePreferenceStatus::Created => {
                return next == EvidencePreferenceStatus::Validated;
            }
            EvidencePreferenceStatus::Validated => {
                match next {
                    EvidencePreferenceStatus::Published | EvidencePreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidencePreferenceStatus::Published => {
                return next == EvidencePreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidencePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence preference", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.port as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidencePreference {
        EvidencePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a evidence preference in status created.
pub fn new_evidence_preference(code: &str, title: &str) -> EvidencePreference {
    EvidencePreference {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        tenant_id: "".to_string(),
        port: 0,
        status: EvidencePreferenceStatus::Created,
    }
}
