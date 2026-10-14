use super::*;

use acme_kit::text;

/// Where a scheduled data export target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDataExportTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledDataExportTargetStatus {
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

impl std::fmt::Display for ScheduledDataExportTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled data export target as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportTarget {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub segment: String,
    pub principal: String,
    pub channel: String,
    pub notes: String,
    pub score: f64,
    pub status: ScheduledDataExportTargetStatus,
}

impl ScheduledDataExportTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDataExportTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDataExportTargetStatus) -> bool {
        match self.status {
            ScheduledDataExportTargetStatus::Draft => {
                match next {
                    ScheduledDataExportTargetStatus::Pending | ScheduledDataExportTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportTargetStatus::Pending => {
                return next == ScheduledDataExportTargetStatus::Suspended;
            }
            ScheduledDataExportTargetStatus::Active => {
                return next == ScheduledDataExportTargetStatus::Closed;
            }
            ScheduledDataExportTargetStatus::Suspended => {
                return next == ScheduledDataExportTargetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDataExportTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled data export target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ScheduledDataExportTarget {
        ScheduledDataExportTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            channel: self.channel.clone(),
            notes: self.notes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled data export target in status draft.
pub fn new_scheduled_data_export_target(code: &str, title: &str) -> ScheduledDataExportTarget {
    ScheduledDataExportTarget {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        segment: "".to_string(),
        principal: "".to_string(),
        channel: "".to_string(),
        notes: "".to_string(),
        score: 0.0,
        status: ScheduledDataExportTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_data_export_target("code-181", "title-720");
        assert_eq!(item.status, ScheduledDataExportTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_data_export_target("code-181", "title-720");
        if item.move_to(ScheduledDataExportTargetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDataExportTargetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_data_export_target("code-181", "title-720");
        item.status = ScheduledDataExportTargetStatus::Closed;
        assert!(
            item.move_to(ScheduledDataExportTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
