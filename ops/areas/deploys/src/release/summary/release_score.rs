use super::*;

use acme_kit::text;

/// Where a release score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReleaseScoreStatus {
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

impl std::fmt::Display for ReleaseScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseScore {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub amount: f64,
    pub burst: i64,
    pub timezone: String,
    pub fingerprint: String,
    pub phone: String,
    pub quota_units: i64,
    pub created_at: i64,
    pub threshold: f64,
    pub status: ReleaseScoreStatus,
}

impl ReleaseScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseScoreStatus) -> bool {
        match self.status {
            ReleaseScoreStatus::New => {
                return next == ReleaseScoreStatus::Queued;
            }
            ReleaseScoreStatus::Queued => {
                return next == ReleaseScoreStatus::Running || next == ReleaseScoreStatus::Done;
            }
            ReleaseScoreStatus::Running => {
                return next == ReleaseScoreStatus::Done || next == ReleaseScoreStatus::Failed;
            }
            ReleaseScoreStatus::Done => {
                return next == ReleaseScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release score {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.created_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseScore {
        ReleaseScore {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            amount: self.amount,
            burst: self.burst,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            phone: self.phone.clone(),
            quota_units: self.quota_units,
            created_at: self.created_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a release score in status new.
pub fn new_release_score(code: &str, label: &str) -> ReleaseScore {
    ReleaseScore {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        amount: 0.0,
        burst: 0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        phone: "".to_string(),
        quota_units: 0,
        created_at: 0,
        threshold: 0.0,
        status: ReleaseScoreStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_release_score("code-8", "label-362");
        assert_eq!(item.status, ReleaseScoreStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_release_score("code-8", "label-362");
        if item.move_to(ReleaseScoreStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseScoreStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_release_score("code-8", "label-362");
        item.status = ReleaseScoreStatus::Failed;
        assert!(item.move_to(ReleaseScoreStatus::New).is_err(), "expected move to fail");
    }
}
