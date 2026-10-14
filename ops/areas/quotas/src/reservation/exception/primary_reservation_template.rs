use super::*;

use acme_kit::text;

/// Where a primary reservation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryReservationTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryReservationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary reservation template as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationTemplate {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub phone: String,
    pub owner_id: String,
    pub status: PrimaryReservationTemplateStatus,
}

impl PrimaryReservationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationTemplateStatus) -> bool {
        match self.status {
            PrimaryReservationTemplateStatus::Proposed => {
                return next == PrimaryReservationTemplateStatus::Withdrawn;
            }
            PrimaryReservationTemplateStatus::Approved => {
                match next {
                    PrimaryReservationTemplateStatus::Rejected | PrimaryReservationTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationTemplateStatus::Rejected => {
                match next {
                    PrimaryReservationTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryReservationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary reservation template", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> PrimaryReservationTemplate {
        PrimaryReservationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary reservation template in status proposed.
pub fn new_primary_reservation_template(code: &str, title: &str) -> PrimaryReservationTemplate {
    PrimaryReservationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        phone: "".to_string(),
        owner_id: "".to_string(),
        status: PrimaryReservationTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_reservation_template("code-201", "title-779");
        assert_eq!(item.status, PrimaryReservationTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_primary_reservation_template("code-201", "title-779");
        if item.move_to(PrimaryReservationTemplateStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationTemplateStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_reservation_template("code-201", "title-779");
        item.status = PrimaryReservationTemplateStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryReservationTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
