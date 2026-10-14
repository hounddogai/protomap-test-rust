use super::*;

use acme_kit::text;

/// Where a evidence claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EvidenceClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EvidenceClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaim {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub cipher: String,
    pub tenant_id: String,
    pub status: EvidenceClaimStatus,
}

impl EvidenceClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceClaimStatus) -> bool {
        match self.status {
            EvidenceClaimStatus::Proposed => {
                return next == EvidenceClaimStatus::Approved;
            }
            EvidenceClaimStatus::Approved => {
                match next {
                    EvidenceClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceClaimStatus::Rejected => {
                return next == EvidenceClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence claim", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> EvidenceClaim {
        EvidenceClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a evidence claim in status proposed.
pub fn new_evidence_claim(code: &str, name: &str) -> EvidenceClaim {
    EvidenceClaim {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        tags: Vec::new(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        status: EvidenceClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_evidence_claim("code-796", "name-122");
        assert_eq!(item.status, EvidenceClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_evidence_claim("code-796", "name-122");
        if item.move_to(EvidenceClaimStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceClaimStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_evidence_claim("code-796", "name-122");
        item.status = EvidenceClaimStatus::Withdrawn;
        assert!(
            item.move_to(EvidenceClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
