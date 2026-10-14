use super::*;

use acme_kit::text;

/// Where a evidence batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EvidenceBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBatch {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub namespace: String,
    pub amount: f64,
    pub retries: i64,
    pub channel: String,
    pub status: EvidenceBatchStatus,
}

impl EvidenceBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceBatchStatus) -> bool {
        match self.status {
            EvidenceBatchStatus::Planned => {
                return next == EvidenceBatchStatus::Cancelled;
            }
            EvidenceBatchStatus::Scheduled => {
                match next {
                    EvidenceBatchStatus::InProgress | EvidenceBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceBatchStatus::InProgress => {
                match next {
                    EvidenceBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceBatchStatus::Completed => {
                match next {
                    EvidenceBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.retries as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceBatch {
        EvidenceBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            namespace: self.namespace.clone(),
            amount: self.amount,
            retries: self.retries,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a evidence batch in status planned.
pub fn new_evidence_batch(code: &str, name: &str) -> EvidenceBatch {
    EvidenceBatch {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        namespace: "".to_string(),
        amount: 0.0,
        retries: 0,
        channel: "".to_string(),
        status: EvidenceBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_batch("code-960", "name-476");
        assert_eq!(item.status, EvidenceBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_evidence_batch("code-960", "name-476");
        if item.move_to(EvidenceBatchStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceBatchStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_batch("code-960", "name-476");
        item.status = EvidenceBatchStatus::Cancelled;
        assert!(
            item.move_to(EvidenceBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
