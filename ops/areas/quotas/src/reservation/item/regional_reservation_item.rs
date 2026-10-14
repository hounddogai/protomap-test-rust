use super::*;

use acme_kit::text;

/// Where a regional reservation item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalReservationItemStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalReservationItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalReservationItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional reservation item, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationItem {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub segment: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub reviewer: String,
    pub ceiling: f64,
    pub capacity: i64,
    pub ratio: f64,
    pub status: RegionalReservationItemStatus,
}

impl RegionalReservationItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalReservationItemStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalReservationItemStatus) -> bool {
        match self.status {
            RegionalReservationItemStatus::Proposed => {
                return next == RegionalReservationItemStatus::Rejected;
            }
            RegionalReservationItemStatus::Approved => {
                return next == RegionalReservationItemStatus::Rejected
                    || next == RegionalReservationItemStatus::Withdrawn;
            }
            RegionalReservationItemStatus::Rejected => {
                return next == RegionalReservationItemStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalReservationItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional reservation item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.sequence as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalReservationItem {
        RegionalReservationItem {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            segment: self.segment.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            capacity: self.capacity,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional reservation item in status proposed.
pub fn new_regional_reservation_item(code: &str, title: &str) -> RegionalReservationItem {
    RegionalReservationItem {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        segment: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        reviewer: "".to_string(),
        ceiling: 0.0,
        capacity: 0,
        ratio: 0.0,
        status: RegionalReservationItemStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_reservation_item("code-222", "title-744");
        assert_eq!(item.status, RegionalReservationItemStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_regional_reservation_item("code-222", "title-744");
        if item.move_to(RegionalReservationItemStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalReservationItemStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_reservation_item("code-222", "title-744");
        item.status = RegionalReservationItemStatus::Withdrawn;
        assert!(
            item.move_to(RegionalReservationItemStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
