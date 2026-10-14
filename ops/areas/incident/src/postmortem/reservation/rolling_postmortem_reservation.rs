use super::*;

use acme_kit::text;

/// Where a rolling postmortem reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPostmortemReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingPostmortemReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingPostmortemReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling postmortem reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemReservation {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub namespace: String,
    pub visible: bool,
    pub priority: i64,
    pub timezone: String,
    pub summary: String,
    pub status: RollingPostmortemReservationStatus,
}

impl RollingPostmortemReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPostmortemReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPostmortemReservationStatus) -> bool {
        match self.status {
            RollingPostmortemReservationStatus::Proposed => {
                match next {
                    RollingPostmortemReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemReservationStatus::Approved => {
                match next {
                    RollingPostmortemReservationStatus::Rejected | RollingPostmortemReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemReservationStatus::Rejected => {
                match next {
                    RollingPostmortemReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingPostmortemReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling postmortem reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        (self.margin * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPostmortemReservation {
        RollingPostmortemReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            visible: self.visible,
            priority: self.priority,
            timezone: self.timezone.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling postmortem reservation in status proposed.
pub fn new_rolling_postmortem_reservation(code: &str, name: &str) -> RollingPostmortemReservation {
    RollingPostmortemReservation {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        visible: false,
        priority: 0,
        timezone: "".to_string(),
        summary: "".to_string(),
        status: RollingPostmortemReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_postmortem_reservation("code-123", "name-210");
        assert_eq!(item.status, RollingPostmortemReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_rolling_postmortem_reservation("code-123", "name-210");
        if item.move_to(RollingPostmortemReservationStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPostmortemReservationStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_postmortem_reservation("code-123", "name-210");
        item.status = RollingPostmortemReservationStatus::Withdrawn;
        assert!(
            item.move_to(RollingPostmortemReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
