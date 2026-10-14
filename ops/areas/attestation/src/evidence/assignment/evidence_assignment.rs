use super::*;

use acme_kit::text;

/// Where a evidence assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EvidenceAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EvidenceAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignment {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub cost: f64,
    pub unit: String,
    pub audience: String,
    pub expires_at: i64,
    pub source: String,
    pub status: EvidenceAssignmentStatus,
}

impl EvidenceAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceAssignmentStatus) -> bool {
        match self.status {
            EvidenceAssignmentStatus::Requested => {
                return next == EvidenceAssignmentStatus::Revoked || next == EvidenceAssignmentStatus::Expired;
            }
            EvidenceAssignmentStatus::Granted => {
                match next {
                    EvidenceAssignmentStatus::Revoked | EvidenceAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceAssignmentStatus::Revoked => {
                return next == EvidenceAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence assignment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceAssignment {
        EvidenceAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            cost: self.cost,
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a evidence assignment in status requested.
pub fn new_evidence_assignment(code: &str, name: &str) -> EvidenceAssignment {
    EvidenceAssignment {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        cost: 0.0,
        unit: "".to_string(),
        audience: "".to_string(),
        expires_at: 0,
        source: "".to_string(),
        status: EvidenceAssignmentStatus::Requested,
    }
}
