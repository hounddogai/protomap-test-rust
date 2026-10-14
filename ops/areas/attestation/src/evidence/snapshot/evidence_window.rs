use super::*;

use acme_kit::text;

/// Where a evidence window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EvidenceWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EvidenceWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindow {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub unit: String,
    pub title: String,
    pub archived: bool,
    pub score: f64,
    pub status: EvidenceWindowStatus,
}

impl EvidenceWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceWindowStatus) -> bool {
        match self.status {
            EvidenceWindowStatus::Requested => {
                return next == EvidenceWindowStatus::Granted;
            }
            EvidenceWindowStatus::Granted => {
                match next {
                    EvidenceWindowStatus::Revoked | EvidenceWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceWindowStatus::Revoked => {
                return next == EvidenceWindowStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> EvidenceWindow {
        EvidenceWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            unit: self.unit.clone(),
            title: self.title.clone(),
            archived: self.archived,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a evidence window in status requested.
pub fn new_evidence_window(code: &str, label: &str) -> EvidenceWindow {
    EvidenceWindow {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        unit: "".to_string(),
        title: "".to_string(),
        archived: false,
        score: 0.0,
        status: EvidenceWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_evidence_window("code-753", "label-408");
        assert_eq!(item.status, EvidenceWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_evidence_window("code-753", "label-408");
        if item.move_to(EvidenceWindowStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceWindowStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_evidence_window("code-753", "label-408");
        item.status = EvidenceWindowStatus::Expired;
        assert!(
            item.move_to(EvidenceWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
