use super::*;

use acme_kit::text;

/// Where a pending peering adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPeeringAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingPeeringAdjustmentStatus {
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

impl std::fmt::Display for PendingPeeringAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending peering adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringAdjustment {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub cluster: String,
    pub notes: String,
    pub amount: f64,
    pub discount: f64,
    pub key_id: String,
    pub status: PendingPeeringAdjustmentStatus,
}

impl PendingPeeringAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPeeringAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPeeringAdjustmentStatus) -> bool {
        match self.status {
            PendingPeeringAdjustmentStatus::New => {
                return next == PendingPeeringAdjustmentStatus::Queued
                    || next == PendingPeeringAdjustmentStatus::Running;
            }
            PendingPeeringAdjustmentStatus::Queued => {
                return next == PendingPeeringAdjustmentStatus::Done || next == PendingPeeringAdjustmentStatus::Failed;
            }
            PendingPeeringAdjustmentStatus::Running => {
                match next {
                    PendingPeeringAdjustmentStatus::Done | PendingPeeringAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPeeringAdjustmentStatus::Done => {
                match next {
                    PendingPeeringAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingPeeringAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending peering adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.discount - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPeeringAdjustment {
        PendingPeeringAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            cluster: self.cluster.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            discount: self.discount,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending peering adjustment in status new.
pub fn new_pending_peering_adjustment(code: &str, name: &str) -> PendingPeeringAdjustment {
    PendingPeeringAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        cluster: "".to_string(),
        notes: "".to_string(),
        amount: 0.0,
        discount: 0.0,
        key_id: "".to_string(),
        status: PendingPeeringAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_peering_adjustment("code-501", "name-576");
        assert_eq!(item.status, PendingPeeringAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_pending_peering_adjustment("code-501", "name-576");
        if item.move_to(PendingPeeringAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPeeringAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_peering_adjustment("code-501", "name-576");
        item.status = PendingPeeringAdjustmentStatus::Failed;
        assert!(
            item.move_to(PendingPeeringAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
