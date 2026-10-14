use super::*;

use acme_kit::text;

/// Where a approval score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalScoreStatus {
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

impl std::fmt::Display for ApprovalScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScore {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub ends_at: i64,
    pub checksum: String,
    pub external_ref: String,
    pub starts_at: i64,
    pub score: f64,
    pub issuer: String,
    pub quota_units: i64,
    pub status: ApprovalScoreStatus,
}

impl ApprovalScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalScoreStatus) -> bool {
        match self.status {
            ApprovalScoreStatus::New => {
                return next == ApprovalScoreStatus::Running;
            }
            ApprovalScoreStatus::Queued => {
                return next == ApprovalScoreStatus::Done || next == ApprovalScoreStatus::Failed;
            }
            ApprovalScoreStatus::Running => {
                return next == ApprovalScoreStatus::Done || next == ApprovalScoreStatus::Failed;
            }
            ApprovalScoreStatus::Done => {
                return next == ApprovalScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval score {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalScore {
        ApprovalScore {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            score: self.score,
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval score in status new.
pub fn new_approval_score(code: &str, label: &str) -> ApprovalScore {
    ApprovalScore {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        ends_at: 0,
        checksum: "".to_string(),
        external_ref: "".to_string(),
        starts_at: 0,
        score: 0.0,
        issuer: "".to_string(),
        quota_units: 0,
        status: ApprovalScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_approval_score("code-990", "label-893");
        assert_eq!(item.status, ApprovalScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_approval_score("code-990", "label-893");
        if item.move_to(ApprovalScoreStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalScoreStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_approval_score("code-990", "label-893");
        item.status = ApprovalScoreStatus::Failed;
        assert!(item.move_to(ApprovalScoreStatus::New).is_err(), "expected move to fail");
    }
}
