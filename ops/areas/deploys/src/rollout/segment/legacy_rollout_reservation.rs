use super::*;

use acme_kit::text;

/// Where a legacy rollout reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyRolloutReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LegacyRolloutReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout reservation as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutReservation {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub margin: f64,
    pub contact_email: String,
    pub locale: String,
    pub reviewer: String,
    pub status: LegacyRolloutReservationStatus,
}

impl LegacyRolloutReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutReservationStatus) -> bool {
        match self.status {
            LegacyRolloutReservationStatus::Planned => {
                return next == LegacyRolloutReservationStatus::Cancelled;
            }
            LegacyRolloutReservationStatus::Scheduled => {
                return next == LegacyRolloutReservationStatus::InProgress;
            }
            LegacyRolloutReservationStatus::InProgress => {
                match next {
                    LegacyRolloutReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRolloutReservationStatus::Completed => {
                return next == LegacyRolloutReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRolloutReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutReservation {
        LegacyRolloutReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy rollout reservation in status planned.
pub fn new_legacy_rollout_reservation(code: &str, title: &str) -> LegacyRolloutReservation {
    LegacyRolloutReservation {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        margin: 0.0,
        contact_email: "".to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        status: LegacyRolloutReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_rollout_reservation("code-34", "title-824");
        assert_eq!(item.status, LegacyRolloutReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_legacy_rollout_reservation("code-34", "title-824");
        if item.move_to(LegacyRolloutReservationStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRolloutReservationStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_rollout_reservation("code-34", "title-824");
        item.status = LegacyRolloutReservationStatus::Cancelled;
        assert!(
            item.move_to(LegacyRolloutReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
