use super::*;

use acme_kit::text;

/// Where a pending approval digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingApprovalDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingApprovalDigestStatus {
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

impl std::fmt::Display for PendingApprovalDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending approval digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalDigest {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub audience: String,
    pub amount: f64,
    pub rate: f64,
    pub duration_minutes: i64,
    pub weight: f64,
    pub status: PendingApprovalDigestStatus,
}

impl PendingApprovalDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingApprovalDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingApprovalDigestStatus) -> bool {
        match self.status {
            PendingApprovalDigestStatus::New => {
                return next == PendingApprovalDigestStatus::Failed;
            }
            PendingApprovalDigestStatus::Queued => {
                return next == PendingApprovalDigestStatus::Running || next == PendingApprovalDigestStatus::Failed;
            }
            PendingApprovalDigestStatus::Running => {
                match next {
                    PendingApprovalDigestStatus::Done | PendingApprovalDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingApprovalDigestStatus::Done => {
                match next {
                    PendingApprovalDigestStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingApprovalDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending approval digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending approval digest", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.weight + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingApprovalDigest {
        PendingApprovalDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            rate: self.rate,
            duration_minutes: self.duration_minutes,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending approval digest in status new.
pub fn new_pending_approval_digest(code: &str, label: &str) -> PendingApprovalDigest {
    PendingApprovalDigest {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        audience: "".to_string(),
        amount: 0.0,
        rate: 0.0,
        duration_minutes: 0,
        weight: 0.0,
        status: PendingApprovalDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_approval_digest("code-890", "label-927");
        assert_eq!(item.status, PendingApprovalDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_pending_approval_digest("code-890", "label-927");
        if item.move_to(PendingApprovalDigestStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingApprovalDigestStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_approval_digest("code-890", "label-927");
        item.status = PendingApprovalDigestStatus::Failed;
        assert!(
            item.move_to(PendingApprovalDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
