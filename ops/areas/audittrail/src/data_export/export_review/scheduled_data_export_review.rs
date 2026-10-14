use super::*;

use acme_kit::text;

/// Where a scheduled data export review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDataExportReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDataExportReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledDataExportReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled data export review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportReview {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub budget: f64,
    pub attempts: i64,
    pub burst: i64,
    pub ends_at: i64,
    pub namespace: String,
    pub status: ScheduledDataExportReviewStatus,
}

impl ScheduledDataExportReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDataExportReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDataExportReviewStatus) -> bool {
        match self.status {
            ScheduledDataExportReviewStatus::Idle => {
                match next {
                    ScheduledDataExportReviewStatus::Ready | ScheduledDataExportReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportReviewStatus::Warming => {
                match next {
                    ScheduledDataExportReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportReviewStatus::Ready => {
                match next {
                    ScheduledDataExportReviewStatus::Draining | ScheduledDataExportReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportReviewStatus::Draining => {
                match next {
                    ScheduledDataExportReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledDataExportReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled data export review {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.budget + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDataExportReview {
        ScheduledDataExportReview {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            budget: self.budget,
            attempts: self.attempts,
            burst: self.burst,
            ends_at: self.ends_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled data export review in status idle.
pub fn new_scheduled_data_export_review(code: &str, title: &str) -> ScheduledDataExportReview {
    ScheduledDataExportReview {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        budget: 0.0,
        attempts: 0,
        burst: 0,
        ends_at: 0,
        namespace: "".to_string(),
        status: ScheduledDataExportReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_data_export_review("code-789", "title-892");
        assert_eq!(item.status, ScheduledDataExportReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_data_export_review("code-789", "title-892");
        if item.move_to(ScheduledDataExportReviewStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDataExportReviewStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_data_export_review("code-789", "title-892");
        item.status = ScheduledDataExportReviewStatus::Stopped;
        assert!(
            item.move_to(ScheduledDataExportReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
