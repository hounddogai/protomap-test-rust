use super::*;

use acme_kit::text;

/// Where a capped verifier assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedVerifierAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedVerifierAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedVerifierAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped verifier assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierAssignment {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub checksum: String,
    pub currency: String,
    pub label: String,
    pub locale: String,
    pub reviewer: String,
    pub namespace: String,
    pub cipher: String,
    pub status: CappedVerifierAssignmentStatus,
}

impl CappedVerifierAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedVerifierAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedVerifierAssignmentStatus) -> bool {
        match self.status {
            CappedVerifierAssignmentStatus::Proposed => {
                match next {
                    CappedVerifierAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierAssignmentStatus::Approved => {
                return next == CappedVerifierAssignmentStatus::Rejected;
            }
            CappedVerifierAssignmentStatus::Rejected => {
                return next == CappedVerifierAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedVerifierAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped verifier assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> CappedVerifierAssignment {
        CappedVerifierAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            currency: self.currency.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped verifier assignment in status proposed.
pub fn new_capped_verifier_assignment(code: &str, name: &str) -> CappedVerifierAssignment {
    CappedVerifierAssignment {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        checksum: "".to_string(),
        currency: "".to_string(),
        label: "".to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        namespace: "".to_string(),
        cipher: "".to_string(),
        status: CappedVerifierAssignmentStatus::Proposed,
    }
}
