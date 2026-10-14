use super::*;

use acme_kit::text;

/// Where a batched region pin reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRegionPinReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedRegionPinReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for BatchedRegionPinReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched region pin reservation as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservation {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub principal: String,
    pub expires_at: i64,
    pub scope: String,
    pub owner_id: String,
    pub tenant_id: String,
    pub priority: i64,
    pub cipher: String,
    pub checksum: String,
    pub status: BatchedRegionPinReservationStatus,
}

impl BatchedRegionPinReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRegionPinReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRegionPinReservationStatus) -> bool {
        match self.status {
            BatchedRegionPinReservationStatus::Open => {
                return next == BatchedRegionPinReservationStatus::Reopened;
            }
            BatchedRegionPinReservationStatus::InReview => {
                match next {
                    BatchedRegionPinReservationStatus::Resolved | BatchedRegionPinReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRegionPinReservationStatus::Resolved => {
                match next {
                    BatchedRegionPinReservationStatus::Reopened | BatchedRegionPinReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRegionPinReservationStatus::Reopened => {
                return next == BatchedRegionPinReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRegionPinReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched region pin reservation {} cannot move from {} to {}",
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
        format!("{} {}", "batched region pin reservation", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.priority as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRegionPinReservation {
        BatchedRegionPinReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            principal: self.principal.clone(),
            expires_at: self.expires_at,
            scope: self.scope.clone(),
            owner_id: self.owner_id.clone(),
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            cipher: self.cipher.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched region pin reservation in status open.
pub fn new_batched_region_pin_reservation(code: &str, name: &str) -> BatchedRegionPinReservation {
    BatchedRegionPinReservation {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        principal: "".to_string(),
        expires_at: 0,
        scope: "".to_string(),
        owner_id: "".to_string(),
        tenant_id: "".to_string(),
        priority: 0,
        cipher: "".to_string(),
        checksum: "".to_string(),
        status: BatchedRegionPinReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_region_pin_reservation("code-551", "name-23");
        assert_eq!(item.status, BatchedRegionPinReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_batched_region_pin_reservation("code-551", "name-23");
        if item.move_to(BatchedRegionPinReservationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRegionPinReservationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_region_pin_reservation("code-551", "name-23");
        item.status = BatchedRegionPinReservationStatus::Archived;
        assert!(
            item.move_to(BatchedRegionPinReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
