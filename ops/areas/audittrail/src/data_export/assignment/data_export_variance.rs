use super::*;

use acme_kit::text;

/// Where a data export variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DataExportVarianceStatus {
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

impl std::fmt::Display for DataExportVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVariance {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub cost: f64,
    pub limit: i64,
    pub issuer: String,
    pub ttl_seconds: i64,
    pub locale: String,
    pub active: bool,
    pub status: DataExportVarianceStatus,
}

impl DataExportVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportVarianceStatus) -> bool {
        match self.status {
            DataExportVarianceStatus::Planned => {
                return next == DataExportVarianceStatus::Completed;
            }
            DataExportVarianceStatus::Scheduled => {
                match next {
                    DataExportVarianceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportVarianceStatus::InProgress => {
                return next == DataExportVarianceStatus::Cancelled;
            }
            DataExportVarianceStatus::Completed => {
                match next {
                    DataExportVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DataExportVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "data export variance", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportVariance {
        DataExportVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            cost: self.cost,
            limit: self.limit,
            issuer: self.issuer.clone(),
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a data export variance in status planned.
pub fn new_data_export_variance(code: &str, label: &str) -> DataExportVariance {
    DataExportVariance {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        cost: 0.0,
        limit: 0,
        issuer: "".to_string(),
        ttl_seconds: 0,
        locale: "".to_string(),
        active: false,
        status: DataExportVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_data_export_variance("code-352", "label-188");
        assert_eq!(item.status, DataExportVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_data_export_variance("code-352", "label-188");
        if item.move_to(DataExportVarianceStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportVarianceStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_data_export_variance("code-352", "label-188");
        item.status = DataExportVarianceStatus::Cancelled;
        assert!(
            item.move_to(DataExportVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
