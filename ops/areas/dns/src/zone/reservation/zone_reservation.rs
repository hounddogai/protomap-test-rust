use super::*;

use acme_kit::text;

/// Where a zone reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone reservation as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReservation {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub revision: i64,
    pub scope: String,
    pub country: String,
    pub archived: bool,
    pub status: ZoneReservationStatus,
}

impl ZoneReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneReservationStatus) -> bool {
        match self.status {
            ZoneReservationStatus::Created => {
                match next {
                    ZoneReservationStatus::Validated | ZoneReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneReservationStatus::Validated => {
                return next == ZoneReservationStatus::Published;
            }
            ZoneReservationStatus::Published => {
                match next {
                    ZoneReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: ZoneReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.revision as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneReservation {
        ZoneReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            revision: self.revision,
            scope: self.scope.clone(),
            country: self.country.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a zone reservation in status created.
pub fn new_zone_reservation(code: &str, label: &str) -> ZoneReservation {
    ZoneReservation {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        revision: 0,
        scope: "".to_string(),
        country: "".to_string(),
        archived: false,
        status: ZoneReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_reservation("code-79", "label-235");
        assert_eq!(item.status, ZoneReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_zone_reservation("code-79", "label-235");
        if item.move_to(ZoneReservationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneReservationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_reservation("code-79", "label-235");
        item.status = ZoneReservationStatus::Retired;
        assert!(
            item.move_to(ZoneReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
