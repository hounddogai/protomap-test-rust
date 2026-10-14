use super::*;

use acme_kit::text;

/// Where a fallback failover reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackFailoverReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackFailoverReservationStatus {
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

impl std::fmt::Display for FallbackFailoverReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback failover reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservation {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub minimum: f64,
    pub visible: bool,
    pub unit: String,
    pub active: bool,
    pub status: FallbackFailoverReservationStatus,
}

impl FallbackFailoverReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackFailoverReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackFailoverReservationStatus) -> bool {
        match self.status {
            FallbackFailoverReservationStatus::Draft => {
                match next {
                    FallbackFailoverReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverReservationStatus::Pending => {
                match next {
                    FallbackFailoverReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverReservationStatus::Active => {
                match next {
                    FallbackFailoverReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackFailoverReservationStatus::Suspended => {
                match next {
                    FallbackFailoverReservationStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackFailoverReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback failover reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.minimum * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackFailoverReservation {
        FallbackFailoverReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            minimum: self.minimum,
            visible: self.visible,
            unit: self.unit.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback failover reservation in status draft.
pub fn new_fallback_failover_reservation(code: &str, name: &str) -> FallbackFailoverReservation {
    FallbackFailoverReservation {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        minimum: 0.0,
        visible: false,
        unit: "".to_string(),
        active: false,
        status: FallbackFailoverReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_failover_reservation("code-987", "name-111");
        assert_eq!(item.status, FallbackFailoverReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_failover_reservation("code-987", "name-111");
        if item.move_to(FallbackFailoverReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackFailoverReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_failover_reservation("code-987", "name-111");
        item.status = FallbackFailoverReservationStatus::Closed;
        assert!(
            item.move_to(FallbackFailoverReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
