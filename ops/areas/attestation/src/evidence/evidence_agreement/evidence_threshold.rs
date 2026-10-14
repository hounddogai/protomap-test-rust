use super::*;

use acme_kit::text;

/// Where a evidence threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EvidenceThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EvidenceThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThreshold {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub capacity: i64,
    pub status: EvidenceThresholdStatus,
}

impl EvidenceThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceThresholdStatus) -> bool {
        match self.status {
            EvidenceThresholdStatus::Proposed => {
                match next {
                    EvidenceThresholdStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceThresholdStatus::Approved => {
                return next == EvidenceThresholdStatus::Rejected || next == EvidenceThresholdStatus::Withdrawn;
            }
            EvidenceThresholdStatus::Rejected => {
                match next {
                    EvidenceThresholdStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EvidenceThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.capacity as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceThreshold {
        EvidenceThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a evidence threshold in status proposed.
pub fn new_evidence_threshold(code: &str, name: &str) -> EvidenceThreshold {
    EvidenceThreshold {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        cipher: "".to_string(),
        capacity: 0,
        status: EvidenceThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_evidence_threshold("code-915", "name-562");
        assert_eq!(item.status, EvidenceThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_evidence_threshold("code-915", "name-562");
        if item.move_to(EvidenceThresholdStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceThresholdStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_evidence_threshold("code-915", "name-562");
        item.status = EvidenceThresholdStatus::Withdrawn;
        assert!(
            item.move_to(EvidenceThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
