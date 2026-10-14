use super::*;

use acme_kit::text;

/// Where a batched approval reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedApprovalReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedApprovalReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedApprovalReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched approval reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservation {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub updated_at: i64,
    pub cluster: String,
    pub scope: String,
    pub limit: i64,
    pub quantity: i64,
    pub checksum: String,
    pub visible: bool,
    pub status: BatchedApprovalReservationStatus,
}

impl BatchedApprovalReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedApprovalReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedApprovalReservationStatus) -> bool {
        match self.status {
            BatchedApprovalReservationStatus::Proposed => {
                return next == BatchedApprovalReservationStatus::Approved
                    || next == BatchedApprovalReservationStatus::Withdrawn;
            }
            BatchedApprovalReservationStatus::Approved => {
                match next {
                    BatchedApprovalReservationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedApprovalReservationStatus::Rejected => {
                return next == BatchedApprovalReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedApprovalReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched approval reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched approval reservation", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.limit as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedApprovalReservation {
        BatchedApprovalReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            limit: self.limit,
            quantity: self.quantity,
            checksum: self.checksum.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched approval reservation in status proposed.
pub fn new_batched_approval_reservation(code: &str, label: &str) -> BatchedApprovalReservation {
    BatchedApprovalReservation {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        updated_at: 0,
        cluster: "".to_string(),
        scope: "".to_string(),
        limit: 0,
        quantity: 0,
        checksum: "".to_string(),
        visible: false,
        status: BatchedApprovalReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_approval_reservation("code-12", "label-835");
        assert_eq!(item.status, BatchedApprovalReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_approval_reservation("code-12", "label-835");
        if item.move_to(BatchedApprovalReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedApprovalReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_approval_reservation("code-12", "label-835");
        item.status = BatchedApprovalReservationStatus::Withdrawn;
        assert!(
            item.move_to(BatchedApprovalReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
