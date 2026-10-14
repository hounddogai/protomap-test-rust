use super::*;

use acme_kit::text;

/// Where a capped evidence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEvidenceAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedEvidenceAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedEvidenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped evidence allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocation {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub pinned: bool,
    pub margin: f64,
    pub active: bool,
    pub status: CappedEvidenceAllocationStatus,
}

impl CappedEvidenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEvidenceAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEvidenceAllocationStatus) -> bool {
        match self.status {
            CappedEvidenceAllocationStatus::Requested => {
                return next == CappedEvidenceAllocationStatus::Granted
                    || next == CappedEvidenceAllocationStatus::Revoked;
            }
            CappedEvidenceAllocationStatus::Granted => {
                return next == CappedEvidenceAllocationStatus::Revoked
                    || next == CappedEvidenceAllocationStatus::Expired;
            }
            CappedEvidenceAllocationStatus::Revoked => {
                match next {
                    CappedEvidenceAllocationStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedEvidenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped evidence allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.capacity as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEvidenceAllocation {
        CappedEvidenceAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            pinned: self.pinned,
            margin: self.margin,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped evidence allocation in status requested.
pub fn new_capped_evidence_allocation(code: &str, name: &str) -> CappedEvidenceAllocation {
    CappedEvidenceAllocation {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        pinned: false,
        margin: 0.0,
        active: false,
        status: CappedEvidenceAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_evidence_allocation("code-31", "name-99");
        assert_eq!(item.status, CappedEvidenceAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_evidence_allocation("code-31", "name-99");
        if item.move_to(CappedEvidenceAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEvidenceAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_evidence_allocation("code-31", "name-99");
        item.status = CappedEvidenceAllocationStatus::Expired;
        assert!(
            item.move_to(CappedEvidenceAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
