use super::*;

use acme_kit::text;

/// Where a evidence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EvidenceExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EvidenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceException {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub retries: i64,
    pub capacity: i64,
    pub category: String,
    pub status: EvidenceExceptionStatus,
}

impl EvidenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceExceptionStatus) -> bool {
        match self.status {
            EvidenceExceptionStatus::Proposed => {
                return next == EvidenceExceptionStatus::Approved || next == EvidenceExceptionStatus::Withdrawn;
            }
            EvidenceExceptionStatus::Approved => {
                return next == EvidenceExceptionStatus::Rejected;
            }
            EvidenceExceptionStatus::Rejected => {
                return next == EvidenceExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceException {
        EvidenceException {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            retries: self.retries,
            capacity: self.capacity,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a evidence exception in status proposed.
pub fn new_evidence_exception(code: &str, label: &str) -> EvidenceException {
    EvidenceException {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        retries: 0,
        capacity: 0,
        category: "".to_string(),
        status: EvidenceExceptionStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_evidence_exception("code-20", "label-480");
        assert_eq!(item.status, EvidenceExceptionStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_evidence_exception("code-20", "label-480");
        if item.move_to(EvidenceExceptionStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceExceptionStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_evidence_exception("code-20", "label-480");
        item.status = EvidenceExceptionStatus::Withdrawn;
        assert!(
            item.move_to(EvidenceExceptionStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
