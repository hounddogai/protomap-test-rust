use super::*;

use acme_kit::text;

/// Where a manual evidence summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEvidenceSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualEvidenceSummaryStatus {
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

impl std::fmt::Display for ManualEvidenceSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual evidence summary as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceSummary {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub visible: bool,
    pub budget: f64,
    pub amount: f64,
    pub floor_value: f64,
    pub status: ManualEvidenceSummaryStatus,
}

impl ManualEvidenceSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEvidenceSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEvidenceSummaryStatus) -> bool {
        match self.status {
            ManualEvidenceSummaryStatus::Planned => {
                return next == ManualEvidenceSummaryStatus::InProgress;
            }
            ManualEvidenceSummaryStatus::Scheduled => {
                return next == ManualEvidenceSummaryStatus::InProgress
                    || next == ManualEvidenceSummaryStatus::Cancelled;
            }
            ManualEvidenceSummaryStatus::InProgress => {
                return next == ManualEvidenceSummaryStatus::Cancelled;
            }
            ManualEvidenceSummaryStatus::Completed => {
                return next == ManualEvidenceSummaryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualEvidenceSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual evidence summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.budget + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEvidenceSummary {
        ManualEvidenceSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            visible: self.visible,
            budget: self.budget,
            amount: self.amount,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual evidence summary in status planned.
pub fn new_manual_evidence_summary(code: &str, title: &str) -> ManualEvidenceSummary {
    ManualEvidenceSummary {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        expires_at: 0,
        visible: false,
        budget: 0.0,
        amount: 0.0,
        floor_value: 0.0,
        status: ManualEvidenceSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_evidence_summary("code-93", "title-260");
        assert_eq!(item.status, ManualEvidenceSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_manual_evidence_summary("code-93", "title-260");
        if item.move_to(ManualEvidenceSummaryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualEvidenceSummaryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_evidence_summary("code-93", "title-260");
        item.status = ManualEvidenceSummaryStatus::Cancelled;
        assert!(
            item.move_to(ManualEvidenceSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
