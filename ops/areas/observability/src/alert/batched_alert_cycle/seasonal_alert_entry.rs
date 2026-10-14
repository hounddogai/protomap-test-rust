use super::*;

use acme_kit::text;

/// Where a seasonal alert entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAlertEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalAlertEntryStatus {
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

impl std::fmt::Display for SeasonalAlertEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal alert entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertEntry {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub cluster: String,
    pub owner_id: String,
    pub status: SeasonalAlertEntryStatus,
}

impl SeasonalAlertEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAlertEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAlertEntryStatus) -> bool {
        match self.status {
            SeasonalAlertEntryStatus::New => {
                return next == SeasonalAlertEntryStatus::Queued;
            }
            SeasonalAlertEntryStatus::Queued => {
                match next {
                    SeasonalAlertEntryStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertEntryStatus::Running => {
                match next {
                    SeasonalAlertEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAlertEntryStatus::Done => {
                return next == SeasonalAlertEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalAlertEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal alert entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> SeasonalAlertEntry {
        SeasonalAlertEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal alert entry in status new.
pub fn new_seasonal_alert_entry(code: &str, name: &str) -> SeasonalAlertEntry {
    SeasonalAlertEntry {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        cluster: "".to_string(),
        owner_id: "".to_string(),
        status: SeasonalAlertEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_alert_entry("code-790", "name-581");
        assert_eq!(item.status, SeasonalAlertEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_alert_entry("code-790", "name-581");
        if item.move_to(SeasonalAlertEntryStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAlertEntryStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_alert_entry("code-790", "name-581");
        item.status = SeasonalAlertEntryStatus::Failed;
        assert!(
            item.move_to(SeasonalAlertEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
