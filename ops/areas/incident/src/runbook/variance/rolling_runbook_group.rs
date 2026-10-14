use super::*;

use acme_kit::text;

/// Where a rolling runbook group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRunbookGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingRunbookGroupStatus {
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

impl std::fmt::Display for RollingRunbookGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling runbook group as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookGroup {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub unit: String,
    pub weight: f64,
    pub description: String,
    pub ends_at: i64,
    pub region: String,
    pub status: RollingRunbookGroupStatus,
}

impl RollingRunbookGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRunbookGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRunbookGroupStatus) -> bool {
        match self.status {
            RollingRunbookGroupStatus::New => {
                return next == RollingRunbookGroupStatus::Done;
            }
            RollingRunbookGroupStatus::Queued => {
                match next {
                    RollingRunbookGroupStatus::Running | RollingRunbookGroupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRunbookGroupStatus::Running => {
                match next {
                    RollingRunbookGroupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRunbookGroupStatus::Done => {
                match next {
                    RollingRunbookGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingRunbookGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling runbook group {} cannot move from {} to {}",
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
        format!("{} {}", "rolling runbook group", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.weight - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRunbookGroup {
        RollingRunbookGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            unit: self.unit.clone(),
            weight: self.weight,
            description: self.description.clone(),
            ends_at: self.ends_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling runbook group in status new.
pub fn new_rolling_runbook_group(code: &str, title: &str) -> RollingRunbookGroup {
    RollingRunbookGroup {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        unit: "".to_string(),
        weight: 0.0,
        description: "".to_string(),
        ends_at: 0,
        region: "".to_string(),
        status: RollingRunbookGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_runbook_group("code-571", "title-686");
        assert_eq!(item.status, RollingRunbookGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_runbook_group("code-571", "title-686");
        if item.move_to(RollingRunbookGroupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRunbookGroupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_runbook_group("code-571", "title-686");
        item.status = RollingRunbookGroupStatus::Failed;
        assert!(
            item.move_to(RollingRunbookGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
