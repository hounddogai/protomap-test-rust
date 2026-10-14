use super::*;

use acme_kit::text;

/// Where a retention metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionMetricStatus {
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

impl std::fmt::Display for RetentionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMetric {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub grace_days: i64,
    pub capacity: i64,
    pub checksum: String,
    pub rate: f64,
    pub limit: i64,
    pub status: RetentionMetricStatus,
}

impl RetentionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionMetricStatus) -> bool {
        match self.status {
            RetentionMetricStatus::Open => {
                return next == RetentionMetricStatus::InReview;
            }
            RetentionMetricStatus::InReview => {
                match next {
                    RetentionMetricStatus::Resolved | RetentionMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionMetricStatus::Resolved => {
                return next == RetentionMetricStatus::Reopened;
            }
            RetentionMetricStatus::Reopened => {
                return next == RetentionMetricStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.rate * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionMetric {
        RetentionMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            capacity: self.capacity,
            checksum: self.checksum.clone(),
            rate: self.rate,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a retention metric in status open.
pub fn new_retention_metric(code: &str, label: &str) -> RetentionMetric {
    RetentionMetric {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        grace_days: 0,
        capacity: 0,
        checksum: "".to_string(),
        rate: 0.0,
        limit: 0,
        status: RetentionMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_retention_metric("code-876", "label-181");
        assert_eq!(item.status, RetentionMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_retention_metric("code-876", "label-181");
        if item.move_to(RetentionMetricStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionMetricStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_retention_metric("code-876", "label-181");
        item.status = RetentionMetricStatus::Archived;
        assert!(
            item.move_to(RetentionMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
