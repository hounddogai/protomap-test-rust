use super::*;

use acme_kit::text;

/// Where a capped overage reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedOverageReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedOverageReservationStatus {
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

impl std::fmt::Display for CappedOverageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped overage reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservation {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub currency: String,
    pub cluster: String,
    pub weight: f64,
    pub cost: f64,
    pub category: String,
    pub key_id: String,
    pub status: CappedOverageReservationStatus,
}

impl CappedOverageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedOverageReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedOverageReservationStatus) -> bool {
        match self.status {
            CappedOverageReservationStatus::Draft => {
                return next == CappedOverageReservationStatus::Active
                    || next == CappedOverageReservationStatus::Closed;
            }
            CappedOverageReservationStatus::Pending => {
                match next {
                    CappedOverageReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedOverageReservationStatus::Active => {
                return next == CappedOverageReservationStatus::Suspended
                    || next == CappedOverageReservationStatus::Closed;
            }
            CappedOverageReservationStatus::Suspended => {
                return next == CappedOverageReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedOverageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped overage reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedOverageReservation {
        CappedOverageReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            weight: self.weight,
            cost: self.cost,
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped overage reservation in status draft.
pub fn new_capped_overage_reservation(code: &str, title: &str) -> CappedOverageReservation {
    CappedOverageReservation {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        currency: "".to_string(),
        cluster: "".to_string(),
        weight: 0.0,
        cost: 0.0,
        category: "".to_string(),
        key_id: "".to_string(),
        status: CappedOverageReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_overage_reservation("code-282", "title-491");
        assert_eq!(item.status, CappedOverageReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_capped_overage_reservation("code-282", "title-491");
        if item.move_to(CappedOverageReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedOverageReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_overage_reservation("code-282", "title-491");
        item.status = CappedOverageReservationStatus::Closed;
        assert!(
            item.move_to(CappedOverageReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
