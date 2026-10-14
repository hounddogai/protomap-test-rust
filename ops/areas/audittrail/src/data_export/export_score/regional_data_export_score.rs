use super::*;

use acme_kit::text;

/// Where a regional data export score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalDataExportScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalDataExportScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RegionalDataExportScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional data export score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScore {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub category: String,
    pub timezone: String,
    pub maximum: f64,
    pub version: i64,
    pub audience: String,
    pub tenant_id: String,
    pub host: String,
    pub burst: i64,
    pub status: RegionalDataExportScoreStatus,
}

impl RegionalDataExportScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalDataExportScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalDataExportScoreStatus) -> bool {
        match self.status {
            RegionalDataExportScoreStatus::New => {
                return next == RegionalDataExportScoreStatus::Running || next == RegionalDataExportScoreStatus::Failed;
            }
            RegionalDataExportScoreStatus::Queued => {
                match next {
                    RegionalDataExportScoreStatus::Running | RegionalDataExportScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportScoreStatus::Running => {
                match next {
                    RegionalDataExportScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalDataExportScoreStatus::Done => {
                return next == RegionalDataExportScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalDataExportScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional data export score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.maximum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalDataExportScore {
        RegionalDataExportScore {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            category: self.category.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            version: self.version,
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional data export score in status new.
pub fn new_regional_data_export_score(code: &str, label: &str) -> RegionalDataExportScore {
    RegionalDataExportScore {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        category: "".to_string(),
        timezone: "".to_string(),
        maximum: 0.0,
        version: 0,
        audience: "".to_string(),
        tenant_id: "".to_string(),
        host: "".to_string(),
        burst: 0,
        status: RegionalDataExportScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_data_export_score("code-846", "label-806");
        assert_eq!(item.status, RegionalDataExportScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_data_export_score("code-846", "label-806");
        if item.move_to(RegionalDataExportScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalDataExportScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_data_export_score("code-846", "label-806");
        item.status = RegionalDataExportScoreStatus::Failed;
        assert!(
            item.move_to(RegionalDataExportScoreStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
