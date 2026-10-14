use super::*;

use acme_kit::text;

/// Where a manual data export segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualDataExportSegmentStatus {
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

impl std::fmt::Display for ManualDataExportSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual data export segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportSegment {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub region: String,
    pub created_at: i64,
    pub weight: f64,
    pub name: String,
    pub status: ManualDataExportSegmentStatus,
}

impl ManualDataExportSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportSegmentStatus) -> bool {
        match self.status {
            ManualDataExportSegmentStatus::Planned => {
                match next {
                    ManualDataExportSegmentStatus::Scheduled | ManualDataExportSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportSegmentStatus::Scheduled => {
                return next == ManualDataExportSegmentStatus::InProgress
                    || next == ManualDataExportSegmentStatus::Completed;
            }
            ManualDataExportSegmentStatus::InProgress => {
                match next {
                    ManualDataExportSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportSegmentStatus::Completed => {
                match next {
                    ManualDataExportSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualDataExportSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual data export segment", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDataExportSegment {
        ManualDataExportSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            region: self.region.clone(),
            created_at: self.created_at,
            weight: self.weight,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual data export segment in status planned.
pub fn new_manual_data_export_segment(code: &str, title: &str) -> ManualDataExportSegment {
    ManualDataExportSegment {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        region: "".to_string(),
        created_at: 0,
        weight: 0.0,
        name: "".to_string(),
        status: ManualDataExportSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_data_export_segment("code-534", "title-223");
        assert_eq!(item.status, ManualDataExportSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_manual_data_export_segment("code-534", "title-223");
        if item.move_to(ManualDataExportSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDataExportSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_data_export_segment("code-534", "title-223");
        item.status = ManualDataExportSegmentStatus::Cancelled;
        assert!(
            item.move_to(ManualDataExportSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
