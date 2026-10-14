use super::*;

use acme_kit::text;

/// Where a data export reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DataExportReservationStatus {
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

impl std::fmt::Display for DataExportReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReservation {
    pub code: String,
    pub label: String,
    pub region: String,
    pub country: String,
    pub maximum: f64,
    pub checksum: String,
    pub status: DataExportReservationStatus,
}

impl DataExportReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportReservationStatus) -> bool {
        match self.status {
            DataExportReservationStatus::Draft => {
                return next == DataExportReservationStatus::Pending;
            }
            DataExportReservationStatus::Pending => {
                return next == DataExportReservationStatus::Suspended;
            }
            DataExportReservationStatus::Active => {
                match next {
                    DataExportReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportReservationStatus::Suspended => {
                return next == DataExportReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> DataExportReservation {
        DataExportReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            country: self.country.clone(),
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a data export reservation in status draft.
pub fn new_data_export_reservation(code: &str, label: &str) -> DataExportReservation {
    DataExportReservation {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        country: "".to_string(),
        maximum: 0.0,
        checksum: "".to_string(),
        status: DataExportReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_data_export_reservation("code-167", "label-173");
        assert_eq!(item.status, DataExportReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_data_export_reservation("code-167", "label-173");
        if item.move_to(DataExportReservationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportReservationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_data_export_reservation("code-167", "label-173");
        item.status = DataExportReservationStatus::Closed;
        assert!(
            item.move_to(DataExportReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
