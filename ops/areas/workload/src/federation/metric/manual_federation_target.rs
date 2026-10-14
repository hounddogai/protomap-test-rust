use super::*;

use acme_kit::text;

/// Where a manual federation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFederationTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualFederationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualFederationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual federation target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTarget {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub notes: String,
    pub tenant_id: String,
    pub batch_size: i64,
    pub discount: f64,
    pub revision: i64,
    pub status: ManualFederationTargetStatus,
}

impl ManualFederationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFederationTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFederationTargetStatus) -> bool {
        match self.status {
            ManualFederationTargetStatus::Created => {
                return next == ManualFederationTargetStatus::Validated
                    || next == ManualFederationTargetStatus::Retired;
            }
            ManualFederationTargetStatus::Validated => {
                return next == ManualFederationTargetStatus::Published
                    || next == ManualFederationTargetStatus::Retired;
            }
            ManualFederationTargetStatus::Published => {
                match next {
                    ManualFederationTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualFederationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual federation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.discount + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFederationTarget {
        ManualFederationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            batch_size: self.batch_size,
            discount: self.discount,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual federation target in status created.
pub fn new_manual_federation_target(code: &str, name: &str) -> ManualFederationTarget {
    ManualFederationTarget {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        batch_size: 0,
        discount: 0.0,
        revision: 0,
        status: ManualFederationTargetStatus::Created,
    }
}
