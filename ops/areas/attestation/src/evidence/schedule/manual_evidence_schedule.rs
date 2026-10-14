use super::*;

use acme_kit::text;

/// Where a manual evidence schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEvidenceScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualEvidenceScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualEvidenceScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual evidence schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceSchedule {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub window_days: i64,
    pub channel: String,
    pub status: ManualEvidenceScheduleStatus,
}

impl ManualEvidenceSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEvidenceScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEvidenceScheduleStatus) -> bool {
        match self.status {
            ManualEvidenceScheduleStatus::Open => {
                match next {
                    ManualEvidenceScheduleStatus::Reopened | ManualEvidenceScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceScheduleStatus::InReview => {
                return next == ManualEvidenceScheduleStatus::Resolved
                    || next == ManualEvidenceScheduleStatus::Archived;
            }
            ManualEvidenceScheduleStatus::Resolved => {
                return next == ManualEvidenceScheduleStatus::Reopened
                    || next == ManualEvidenceScheduleStatus::Archived;
            }
            ManualEvidenceScheduleStatus::Reopened => {
                return next == ManualEvidenceScheduleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualEvidenceScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual evidence schedule {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.created_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEvidenceSchedule {
        ManualEvidenceSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            window_days: self.window_days,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual evidence schedule in status open.
pub fn new_manual_evidence_schedule(code: &str, title: &str) -> ManualEvidenceSchedule {
    ManualEvidenceSchedule {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        window_days: 0,
        channel: "".to_string(),
        status: ManualEvidenceScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_evidence_schedule("code-753", "title-758");
        assert_eq!(item.status, ManualEvidenceScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_evidence_schedule("code-753", "title-758");
        if item.move_to(ManualEvidenceScheduleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualEvidenceScheduleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_evidence_schedule("code-753", "title-758");
        item.status = ManualEvidenceScheduleStatus::Archived;
        assert!(
            item.move_to(ManualEvidenceScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
