use super::*;

use acme_kit::text;

/// Where a regional data export reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDataExportReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalDataExportReservationStatus {
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

impl std::fmt::Display for RegionalDataExportReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional data export reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservation {
    pub code: String,
    pub label: String,
    pub name: String,
    pub capacity: i64,
    pub contact_email: String,
    pub owner_id: String,
    pub audience: String,
    pub status: RegionalDataExportReservationStatus,
}

impl RegionalDataExportReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDataExportReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDataExportReservationStatus) -> bool {
        match self.status {
            RegionalDataExportReservationStatus::Draft => {
                return next == RegionalDataExportReservationStatus::Active;
            }
            RegionalDataExportReservationStatus::Pending => {
                match next {
                    RegionalDataExportReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportReservationStatus::Active => {
                return next == RegionalDataExportReservationStatus::Suspended
                    || next == RegionalDataExportReservationStatus::Closed;
            }
            RegionalDataExportReservationStatus::Suspended => {
                match next {
                    RegionalDataExportReservationStatus::Closed => {
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

    pub fn move_to(&mut self, next: RegionalDataExportReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional data export reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional data export reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> RegionalDataExportReservation {
        RegionalDataExportReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional data export reservation in status draft.
pub fn new_regional_data_export_reservation(code: &str, label: &str) -> RegionalDataExportReservation {
    RegionalDataExportReservation {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        capacity: 0,
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        audience: "".to_string(),
        status: RegionalDataExportReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_data_export_reservation("code-441", "label-818");
        assert_eq!(item.status, RegionalDataExportReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_regional_data_export_reservation("code-441", "label-818");
        if item.move_to(RegionalDataExportReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDataExportReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_data_export_reservation("code-441", "label-818");
        item.status = RegionalDataExportReservationStatus::Closed;
        assert!(
            item.move_to(RegionalDataExportReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
