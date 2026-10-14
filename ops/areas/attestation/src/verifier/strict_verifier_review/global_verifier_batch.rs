use super::*;

use acme_kit::text;

/// Where a global verifier batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalVerifierBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalVerifierBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalVerifierBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global verifier batch as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierBatch {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub ceiling: f64,
    pub host: String,
    pub title: String,
    pub score: f64,
    pub discount: f64,
    pub phone: String,
    pub status: GlobalVerifierBatchStatus,
}

impl GlobalVerifierBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalVerifierBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalVerifierBatchStatus) -> bool {
        match self.status {
            GlobalVerifierBatchStatus::Proposed => {
                return next == GlobalVerifierBatchStatus::Rejected;
            }
            GlobalVerifierBatchStatus::Approved => {
                match next {
                    GlobalVerifierBatchStatus::Rejected | GlobalVerifierBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalVerifierBatchStatus::Rejected => {
                match next {
                    GlobalVerifierBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GlobalVerifierBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global verifier batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global verifier batch", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.discount * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalVerifierBatch {
        GlobalVerifierBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            ceiling: self.ceiling,
            host: self.host.clone(),
            title: self.title.clone(),
            score: self.score,
            discount: self.discount,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global verifier batch in status proposed.
pub fn new_global_verifier_batch(code: &str, label: &str) -> GlobalVerifierBatch {
    GlobalVerifierBatch {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        ceiling: 0.0,
        host: "".to_string(),
        title: "".to_string(),
        score: 0.0,
        discount: 0.0,
        phone: "".to_string(),
        status: GlobalVerifierBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_verifier_batch("code-839", "label-231");
        assert_eq!(item.status, GlobalVerifierBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_global_verifier_batch("code-839", "label-231");
        if item.move_to(GlobalVerifierBatchStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalVerifierBatchStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_verifier_batch("code-839", "label-231");
        item.status = GlobalVerifierBatchStatus::Withdrawn;
        assert!(
            item.move_to(GlobalVerifierBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
