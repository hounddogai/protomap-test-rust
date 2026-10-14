use super::*;

use acme_kit::text;

/// Where a batched data export reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDataExportReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedDataExportReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedDataExportReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched data export reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservation {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub amount: f64,
    pub rate: f64,
    pub limit: i64,
    pub maximum: f64,
    pub principal: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub approved: bool,
    pub status: BatchedDataExportReservationStatus,
}

impl BatchedDataExportReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDataExportReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDataExportReservationStatus) -> bool {
        match self.status {
            BatchedDataExportReservationStatus::Requested => {
                return next == BatchedDataExportReservationStatus::Granted;
            }
            BatchedDataExportReservationStatus::Granted => {
                match next {
                    BatchedDataExportReservationStatus::Revoked | BatchedDataExportReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDataExportReservationStatus::Revoked => {
                match next {
                    BatchedDataExportReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedDataExportReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched data export reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.amount * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDataExportReservation {
        BatchedDataExportReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            amount: self.amount,
            rate: self.rate,
            limit: self.limit,
            maximum: self.maximum,
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched data export reservation in status requested.
pub fn new_batched_data_export_reservation(code: &str, title: &str) -> BatchedDataExportReservation {
    BatchedDataExportReservation {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        amount: 0.0,
        rate: 0.0,
        limit: 0,
        maximum: 0.0,
        principal: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        approved: false,
        status: BatchedDataExportReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_data_export_reservation("code-8", "title-397");
        assert_eq!(item.status, BatchedDataExportReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_data_export_reservation("code-8", "title-397");
        if item.move_to(BatchedDataExportReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDataExportReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_data_export_reservation("code-8", "title-397");
        item.status = BatchedDataExportReservationStatus::Expired;
        assert!(
            item.move_to(BatchedDataExportReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
