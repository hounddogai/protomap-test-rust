use super::*;

use acme_kit::text;

/// Where a silence review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SilenceReviewStatus {
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

impl std::fmt::Display for SilenceReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReview {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub expires_at: i64,
    pub quota_units: i64,
    pub batch_size: i64,
    pub status: SilenceReviewStatus,
}

impl SilenceReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceReviewStatus) -> bool {
        match self.status {
            SilenceReviewStatus::Planned => {
                return next == SilenceReviewStatus::Scheduled || next == SilenceReviewStatus::Cancelled;
            }
            SilenceReviewStatus::Scheduled => {
                match next {
                    SilenceReviewStatus::InProgress | SilenceReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceReviewStatus::InProgress => {
                match next {
                    SilenceReviewStatus::Completed | SilenceReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceReviewStatus::Completed => {
                match next {
                    SilenceReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SilenceReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence review {} cannot move from {} to {}",
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

    pub fn projected_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceReview {
        SilenceReview {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a silence review in status planned.
pub fn new_silence_review(code: &str, title: &str) -> SilenceReview {
    SilenceReview {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        expires_at: 0,
        quota_units: 0,
        batch_size: 0,
        status: SilenceReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_silence_review("code-570", "title-959");
        assert_eq!(item.status, SilenceReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_silence_review("code-570", "title-959");
        if item.move_to(SilenceReviewStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceReviewStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_silence_review("code-570", "title-959");
        item.status = SilenceReviewStatus::Cancelled;
        assert!(
            item.move_to(SilenceReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
