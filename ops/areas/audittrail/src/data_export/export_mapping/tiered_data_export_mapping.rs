use super::*;

use acme_kit::text;

/// Where a tiered data export mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDataExportMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredDataExportMappingStatus {
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

impl std::fmt::Display for TieredDataExportMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered data export mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMapping {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub minimum: f64,
    pub status: TieredDataExportMappingStatus,
}

impl TieredDataExportMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDataExportMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDataExportMappingStatus) -> bool {
        match self.status {
            TieredDataExportMappingStatus::Planned => {
                match next {
                    TieredDataExportMappingStatus::InProgress | TieredDataExportMappingStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDataExportMappingStatus::Scheduled => {
                return next == TieredDataExportMappingStatus::Cancelled;
            }
            TieredDataExportMappingStatus::InProgress => {
                return next == TieredDataExportMappingStatus::Completed;
            }
            TieredDataExportMappingStatus::Completed => {
                match next {
                    TieredDataExportMappingStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredDataExportMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered data export mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDataExportMapping {
        TieredDataExportMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered data export mapping in status planned.
pub fn new_tiered_data_export_mapping(code: &str, title: &str) -> TieredDataExportMapping {
    TieredDataExportMapping {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        ttl_seconds: 0,
        revision: 0,
        minimum: 0.0,
        status: TieredDataExportMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_data_export_mapping("code-82", "title-670");
        assert_eq!(item.status, TieredDataExportMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_tiered_data_export_mapping("code-82", "title-670");
        if item.move_to(TieredDataExportMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDataExportMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_data_export_mapping("code-82", "title-670");
        item.status = TieredDataExportMappingStatus::Cancelled;
        assert!(
            item.move_to(TieredDataExportMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
