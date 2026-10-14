use super::*;

use acme_kit::text;

/// Where a rolling data export mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDataExportMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingDataExportMappingStatus {
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

impl std::fmt::Display for RollingDataExportMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling data export mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDataExportMapping {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub priority: i64,
    pub ends_at: i64,
    pub status: RollingDataExportMappingStatus,
}

impl RollingDataExportMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDataExportMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDataExportMappingStatus) -> bool {
        match self.status {
            RollingDataExportMappingStatus::Planned => {
                return next == RollingDataExportMappingStatus::InProgress
                    || next == RollingDataExportMappingStatus::Completed;
            }
            RollingDataExportMappingStatus::Scheduled => {
                match next {
                    RollingDataExportMappingStatus::InProgress | RollingDataExportMappingStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportMappingStatus::InProgress => {
                match next {
                    RollingDataExportMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDataExportMappingStatus::Completed => {
                return next == RollingDataExportMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDataExportMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling data export mapping {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDataExportMapping {
        RollingDataExportMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            priority: self.priority,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling data export mapping in status planned.
pub fn new_rolling_data_export_mapping(code: &str, name: &str) -> RollingDataExportMapping {
    RollingDataExportMapping {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        priority: 0,
        ends_at: 0,
        status: RollingDataExportMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_data_export_mapping("code-187", "name-731");
        assert_eq!(item.status, RollingDataExportMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_rolling_data_export_mapping("code-187", "name-731");
        if item.move_to(RollingDataExportMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDataExportMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_data_export_mapping("code-187", "name-731");
        item.status = RollingDataExportMappingStatus::Cancelled;
        assert!(
            item.move_to(RollingDataExportMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
