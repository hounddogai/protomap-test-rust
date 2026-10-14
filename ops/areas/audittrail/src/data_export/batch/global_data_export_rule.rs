use super::*;

use acme_kit::text;

/// Where a global data export rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDataExportRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalDataExportRuleStatus {
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

impl std::fmt::Display for GlobalDataExportRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global data export rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportRule {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub priority: i64,
    pub revision: i64,
    pub maximum: f64,
    pub host: String,
    pub port: i64,
    pub pinned: bool,
    pub status: GlobalDataExportRuleStatus,
}

impl GlobalDataExportRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDataExportRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDataExportRuleStatus) -> bool {
        match self.status {
            GlobalDataExportRuleStatus::Planned => {
                match next {
                    GlobalDataExportRuleStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDataExportRuleStatus::Scheduled => {
                match next {
                    GlobalDataExportRuleStatus::Completed | GlobalDataExportRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDataExportRuleStatus::InProgress => {
                match next {
                    GlobalDataExportRuleStatus::Completed | GlobalDataExportRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalDataExportRuleStatus::Completed => {
                return next == GlobalDataExportRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalDataExportRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global data export rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global data export rule", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDataExportRule {
        GlobalDataExportRule {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            priority: self.priority,
            revision: self.revision,
            maximum: self.maximum,
            host: self.host.clone(),
            port: self.port,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global data export rule in status planned.
pub fn new_global_data_export_rule(code: &str, label: &str) -> GlobalDataExportRule {
    GlobalDataExportRule {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        priority: 0,
        revision: 0,
        maximum: 0.0,
        host: "".to_string(),
        port: 0,
        pinned: false,
        status: GlobalDataExportRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_data_export_rule("code-701", "label-817");
        assert_eq!(item.status, GlobalDataExportRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_data_export_rule("code-701", "label-817");
        if item.move_to(GlobalDataExportRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalDataExportRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_data_export_rule("code-701", "label-817");
        item.status = GlobalDataExportRuleStatus::Cancelled;
        assert!(
            item.move_to(GlobalDataExportRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
