use super::*;

use acme_kit::text;

/// Where a data export estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DataExportEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DataExportEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data export estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEstimate {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub budget: f64,
    pub notes: String,
    pub contact_email: String,
    pub ends_at: i64,
    pub status: DataExportEstimateStatus,
}

impl DataExportEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportEstimateStatus) -> bool {
        match self.status {
            DataExportEstimateStatus::Requested => {
                return next == DataExportEstimateStatus::Granted || next == DataExportEstimateStatus::Revoked;
            }
            DataExportEstimateStatus::Granted => {
                return next == DataExportEstimateStatus::Revoked || next == DataExportEstimateStatus::Expired;
            }
            DataExportEstimateStatus::Revoked => {
                match next {
                    DataExportEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: DataExportEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export estimate {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "data export estimate", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportEstimate {
        DataExportEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            budget: self.budget,
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a data export estimate in status requested.
pub fn new_data_export_estimate(code: &str, title: &str) -> DataExportEstimate {
    DataExportEstimate {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        budget: 0.0,
        notes: "".to_string(),
        contact_email: "".to_string(),
        ends_at: 0,
        status: DataExportEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_data_export_estimate("code-198", "title-691");
        assert_eq!(item.status, DataExportEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_data_export_estimate("code-198", "title-691");
        if item.move_to(DataExportEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_data_export_estimate("code-198", "title-691");
        item.status = DataExportEstimateStatus::Expired;
        assert!(
            item.move_to(DataExportEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
