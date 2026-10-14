use super::*;

use acme_kit::text;

/// Where a rolling runbook summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRunbookSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingRunbookSummaryStatus {
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

impl std::fmt::Display for RollingRunbookSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling runbook summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookSummary {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub rotation_days: i64,
    pub owner_id: String,
    pub weight: f64,
    pub phone: String,
    pub updated_at: i64,
    pub status: RollingRunbookSummaryStatus,
}

impl RollingRunbookSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRunbookSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRunbookSummaryStatus) -> bool {
        match self.status {
            RollingRunbookSummaryStatus::New => {
                match next {
                    RollingRunbookSummaryStatus::Done | RollingRunbookSummaryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRunbookSummaryStatus::Queued => {
                return next == RollingRunbookSummaryStatus::Running;
            }
            RollingRunbookSummaryStatus::Running => {
                return next == RollingRunbookSummaryStatus::Done || next == RollingRunbookSummaryStatus::Failed;
            }
            RollingRunbookSummaryStatus::Done => {
                match next {
                    RollingRunbookSummaryStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingRunbookSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling runbook summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.weight + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRunbookSummary {
        RollingRunbookSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            rotation_days: self.rotation_days,
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            phone: self.phone.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling runbook summary in status new.
pub fn new_rolling_runbook_summary(code: &str, title: &str) -> RollingRunbookSummary {
    RollingRunbookSummary {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        rotation_days: 0,
        owner_id: "".to_string(),
        weight: 0.0,
        phone: "".to_string(),
        updated_at: 0,
        status: RollingRunbookSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_runbook_summary("code-929", "title-361");
        assert_eq!(item.status, RollingRunbookSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_runbook_summary("code-929", "title-361");
        if item.move_to(RollingRunbookSummaryStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRunbookSummaryStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_runbook_summary("code-929", "title-361");
        item.status = RollingRunbookSummaryStatus::Failed;
        assert!(
            item.move_to(RollingRunbookSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
