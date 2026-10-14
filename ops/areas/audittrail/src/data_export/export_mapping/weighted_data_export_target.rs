use super::*;

use acme_kit::text;

/// Where a weighted data export target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedDataExportTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedDataExportTargetStatus {
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

impl std::fmt::Display for WeightedDataExportTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted data export target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportTarget {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub phone: String,
    pub cost: f64,
    pub host: String,
    pub cluster: String,
    pub category: String,
    pub status: WeightedDataExportTargetStatus,
}

impl WeightedDataExportTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedDataExportTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedDataExportTargetStatus) -> bool {
        match self.status {
            WeightedDataExportTargetStatus::Planned => {
                return next == WeightedDataExportTargetStatus::Completed
                    || next == WeightedDataExportTargetStatus::Cancelled;
            }
            WeightedDataExportTargetStatus::Scheduled => {
                match next {
                    WeightedDataExportTargetStatus::InProgress | WeightedDataExportTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDataExportTargetStatus::InProgress => {
                return next == WeightedDataExportTargetStatus::Completed
                    || next == WeightedDataExportTargetStatus::Cancelled;
            }
            WeightedDataExportTargetStatus::Completed => {
                return next == WeightedDataExportTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedDataExportTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted data export target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted data export target", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.priority as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedDataExportTarget {
        WeightedDataExportTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            phone: self.phone.clone(),
            cost: self.cost,
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a weighted data export target in status planned.
pub fn new_weighted_data_export_target(code: &str, name: &str) -> WeightedDataExportTarget {
    WeightedDataExportTarget {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        phone: "".to_string(),
        cost: 0.0,
        host: "".to_string(),
        cluster: "".to_string(),
        category: "".to_string(),
        status: WeightedDataExportTargetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_data_export_target("code-233", "name-55");
        assert_eq!(item.status, WeightedDataExportTargetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_data_export_target("code-233", "name-55");
        if item.move_to(WeightedDataExportTargetStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedDataExportTargetStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_data_export_target("code-233", "name-55");
        item.status = WeightedDataExportTargetStatus::Cancelled;
        assert!(
            item.move_to(WeightedDataExportTargetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
