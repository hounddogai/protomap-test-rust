use super::*;

use acme_kit::text;

/// Where a batched reservation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReservationPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedReservationPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedReservationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched reservation policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicy {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub key_id: String,
    pub port: i64,
    pub threshold: f64,
    pub reviewer: String,
    pub status: BatchedReservationPolicyStatus,
}

impl BatchedReservationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReservationPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReservationPolicyStatus) -> bool {
        match self.status {
            BatchedReservationPolicyStatus::Proposed => {
                return next == BatchedReservationPolicyStatus::Approved
                    || next == BatchedReservationPolicyStatus::Rejected;
            }
            BatchedReservationPolicyStatus::Approved => {
                return next == BatchedReservationPolicyStatus::Rejected
                    || next == BatchedReservationPolicyStatus::Withdrawn;
            }
            BatchedReservationPolicyStatus::Rejected => {
                return next == BatchedReservationPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedReservationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched reservation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedReservationPolicy {
        BatchedReservationPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            port: self.port,
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched reservation policy in status proposed.
pub fn new_batched_reservation_policy(code: &str, title: &str) -> BatchedReservationPolicy {
    BatchedReservationPolicy {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        key_id: "".to_string(),
        port: 0,
        threshold: 0.0,
        reviewer: "".to_string(),
        status: BatchedReservationPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_reservation_policy("code-941", "title-82");
        assert_eq!(item.status, BatchedReservationPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_reservation_policy("code-941", "title-82");
        if item.move_to(BatchedReservationPolicyStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReservationPolicyStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_reservation_policy("code-941", "title-82");
        item.status = BatchedReservationPolicyStatus::Withdrawn;
        assert!(
            item.move_to(BatchedReservationPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
