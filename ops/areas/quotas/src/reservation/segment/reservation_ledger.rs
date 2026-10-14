use super::*;

use acme_kit::text;

/// Where a reservation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReservationLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReservationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation ledger as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationLedger {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub owner_id: String,
    pub version: i64,
    pub active: bool,
    pub tags: Vec<String>,
    pub burst: i64,
    pub region: String,
    pub maximum: f64,
    pub status: ReservationLedgerStatus,
}

impl ReservationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationLedgerStatus) -> bool {
        match self.status {
            ReservationLedgerStatus::Created => {
                return next == ReservationLedgerStatus::Published;
            }
            ReservationLedgerStatus::Validated => {
                return next == ReservationLedgerStatus::Published || next == ReservationLedgerStatus::Retired;
            }
            ReservationLedgerStatus::Published => {
                return next == ReservationLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation ledger", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationLedger {
        ReservationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            owner_id: self.owner_id.clone(),
            version: self.version,
            active: self.active,
            tags: self.tags.clone(),
            burst: self.burst,
            region: self.region.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a reservation ledger in status created.
pub fn new_reservation_ledger(code: &str, name: &str) -> ReservationLedger {
    ReservationLedger {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        owner_id: "".to_string(),
        version: 0,
        active: false,
        tags: Vec::new(),
        burst: 0,
        region: "".to_string(),
        maximum: 0.0,
        status: ReservationLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_reservation_ledger("code-423", "name-829");
        assert_eq!(item.status, ReservationLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_reservation_ledger("code-423", "name-829");
        if item.move_to(ReservationLedgerStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationLedgerStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_reservation_ledger("code-423", "name-829");
        item.status = ReservationLedgerStatus::Retired;
        assert!(
            item.move_to(ReservationLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
