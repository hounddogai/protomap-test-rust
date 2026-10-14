use super::*;

use acme_kit::text;

/// Where a freeze reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FreezeReservationStatus {
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

impl std::fmt::Display for FreezeReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze reservation as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeReservation {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub capacity: i64,
    pub margin: f64,
    pub tags: Vec<String>,
    pub currency: String,
    pub tenant_id: String,
    pub visible: bool,
    pub amount: f64,
    pub status: FreezeReservationStatus,
}

impl FreezeReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeReservationStatus) -> bool {
        match self.status {
            FreezeReservationStatus::Planned => {
                match next {
                    FreezeReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeReservationStatus::Scheduled => {
                match next {
                    FreezeReservationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeReservationStatus::InProgress => {
                return next == FreezeReservationStatus::Completed || next == FreezeReservationStatus::Cancelled;
            }
            FreezeReservationStatus::Completed => {
                return next == FreezeReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze reservation {} cannot move from {} to {}",
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
        format!("{} {}", "freeze reservation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.margin - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeReservation {
        FreezeReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            capacity: self.capacity,
            margin: self.margin,
            tags: self.tags.clone(),
            currency: self.currency.clone(),
            tenant_id: self.tenant_id.clone(),
            visible: self.visible,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze reservation in status planned.
pub fn new_freeze_reservation(code: &str, name: &str) -> FreezeReservation {
    FreezeReservation {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        capacity: 0,
        margin: 0.0,
        tags: Vec::new(),
        currency: "".to_string(),
        tenant_id: "".to_string(),
        visible: false,
        amount: 0.0,
        status: FreezeReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_freeze_reservation("code-234", "name-180");
        assert_eq!(item.status, FreezeReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_freeze_reservation("code-234", "name-180");
        if item.move_to(FreezeReservationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeReservationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_freeze_reservation("code-234", "name-180");
        item.status = FreezeReservationStatus::Cancelled;
        assert!(
            item.move_to(FreezeReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
