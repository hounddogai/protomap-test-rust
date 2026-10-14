use super::*;

use acme_kit::text;

/// Where a evidence policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidencePolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EvidencePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for EvidencePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence policy as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicy {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub principal: String,
    pub external_ref: String,
    pub rotation_days: i64,
    pub maximum: f64,
    pub weight: f64,
    pub status: EvidencePolicyStatus,
}

impl EvidencePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidencePolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidencePolicyStatus) -> bool {
        match self.status {
            EvidencePolicyStatus::New => {
                match next {
                    EvidencePolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidencePolicyStatus::Queued => {
                return next == EvidencePolicyStatus::Done || next == EvidencePolicyStatus::Failed;
            }
            EvidencePolicyStatus::Running => {
                return next == EvidencePolicyStatus::Failed;
            }
            EvidencePolicyStatus::Done => {
                match next {
                    EvidencePolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: EvidencePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidencePolicy {
        EvidencePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a evidence policy in status new.
pub fn new_evidence_policy(code: &str, name: &str) -> EvidencePolicy {
    EvidencePolicy {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        principal: "".to_string(),
        external_ref: "".to_string(),
        rotation_days: 0,
        maximum: 0.0,
        weight: 0.0,
        status: EvidencePolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_evidence_policy("code-347", "name-323");
        assert_eq!(item.status, EvidencePolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_evidence_policy("code-347", "name-323");
        if item.move_to(EvidencePolicyStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidencePolicyStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_evidence_policy("code-347", "name-323");
        item.status = EvidencePolicyStatus::Failed;
        assert!(
            item.move_to(EvidencePolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
