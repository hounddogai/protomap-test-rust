use super::*;

use acme_kit::text;

/// Where a retention variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionVarianceStatus {
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

impl std::fmt::Display for RetentionVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVariance {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub segment: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub category: String,
    pub status: RetentionVarianceStatus,
}

impl RetentionVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionVarianceStatus) -> bool {
        match self.status {
            RetentionVarianceStatus::New => {
                return next == RetentionVarianceStatus::Running || next == RetentionVarianceStatus::Failed;
            }
            RetentionVarianceStatus::Queued => {
                match next {
                    RetentionVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionVarianceStatus::Running => {
                return next == RetentionVarianceStatus::Done || next == RetentionVarianceStatus::Failed;
            }
            RetentionVarianceStatus::Done => {
                return next == RetentionVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "retention variance", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.starts_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionVariance {
        RetentionVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            segment: self.segment.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a retention variance in status new.
pub fn new_retention_variance(code: &str, label: &str) -> RetentionVariance {
    RetentionVariance {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        segment: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        category: "".to_string(),
        status: RetentionVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_variance("code-239", "label-689");
        assert_eq!(item.status, RetentionVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_retention_variance("code-239", "label-689");
        if item.move_to(RetentionVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_variance("code-239", "label-689");
        item.status = RetentionVarianceStatus::Failed;
        assert!(
            item.move_to(RetentionVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
