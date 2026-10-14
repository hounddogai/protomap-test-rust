use super::*;

use acme_kit::text;

/// Where a capped verifier schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedVerifierScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedVerifierScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedVerifierScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped verifier schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierSchedule {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub sequence: i64,
    pub status: CappedVerifierScheduleStatus,
}

impl CappedVerifierSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedVerifierScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedVerifierScheduleStatus) -> bool {
        match self.status {
            CappedVerifierScheduleStatus::Draft => {
                match next {
                    CappedVerifierScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierScheduleStatus::Pending => {
                match next {
                    CappedVerifierScheduleStatus::Active | CappedVerifierScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedVerifierScheduleStatus::Active => {
                return next == CappedVerifierScheduleStatus::Suspended || next == CappedVerifierScheduleStatus::Closed;
            }
            CappedVerifierScheduleStatus::Suspended => {
                return next == CappedVerifierScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedVerifierScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped verifier schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.batch_size as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedVerifierSchedule {
        CappedVerifierSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped verifier schedule in status draft.
pub fn new_capped_verifier_schedule(code: &str, label: &str) -> CappedVerifierSchedule {
    CappedVerifierSchedule {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        threshold: 0.0,
        batch_size: 0,
        sequence: 0,
        status: CappedVerifierScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_verifier_schedule("code-686", "label-229");
        assert_eq!(item.status, CappedVerifierScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_capped_verifier_schedule("code-686", "label-229");
        if item.move_to(CappedVerifierScheduleStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedVerifierScheduleStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_verifier_schedule("code-686", "label-229");
        item.status = CappedVerifierScheduleStatus::Closed;
        assert!(
            item.move_to(CappedVerifierScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
