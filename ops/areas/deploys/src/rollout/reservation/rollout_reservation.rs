use super::*;

use acme_kit::text;

/// Where a rollout reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RolloutReservationStatus {
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

impl std::fmt::Display for RolloutReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservation {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub amount: f64,
    pub ceiling: f64,
    pub floor_value: f64,
    pub locale: String,
    pub ends_at: i64,
    pub weight: f64,
    pub status: RolloutReservationStatus,
}

impl RolloutReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutReservationStatus) -> bool {
        match self.status {
            RolloutReservationStatus::Draft => {
                match next {
                    RolloutReservationStatus::Active | RolloutReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutReservationStatus::Pending => {
                return next == RolloutReservationStatus::Active || next == RolloutReservationStatus::Suspended;
            }
            RolloutReservationStatus::Active => {
                match next {
                    RolloutReservationStatus::Suspended | RolloutReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutReservationStatus::Suspended => {
                return next == RolloutReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollout reservation", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ends_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutReservation {
        RolloutReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            amount: self.amount,
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rollout reservation in status draft.
pub fn new_rollout_reservation(code: &str, label: &str) -> RolloutReservation {
    RolloutReservation {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        amount: 0.0,
        ceiling: 0.0,
        floor_value: 0.0,
        locale: "".to_string(),
        ends_at: 0,
        weight: 0.0,
        status: RolloutReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollout_reservation("code-84", "label-831");
        assert_eq!(item.status, RolloutReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rollout_reservation("code-84", "label-831");
        if item.move_to(RolloutReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollout_reservation("code-84", "label-831");
        item.status = RolloutReservationStatus::Closed;
        assert!(
            item.move_to(RolloutReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
