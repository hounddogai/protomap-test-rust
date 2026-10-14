use super::*;

use acme_kit::text;

/// Where a retention adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionAdjustmentStatus {
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

impl std::fmt::Display for RetentionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustment {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub score: f64,
    pub cluster: String,
    pub reviewer: String,
    pub status: RetentionAdjustmentStatus,
}

impl RetentionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionAdjustmentStatus) -> bool {
        match self.status {
            RetentionAdjustmentStatus::Open => {
                match next {
                    RetentionAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAdjustmentStatus::InReview => {
                match next {
                    RetentionAdjustmentStatus::Resolved | RetentionAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAdjustmentStatus::Resolved => {
                return next == RetentionAdjustmentStatus::Reopened;
            }
            RetentionAdjustmentStatus::Reopened => {
                return next == RetentionAdjustmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.starts_at as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionAdjustment {
        RetentionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            score: self.score,
            cluster: self.cluster.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a retention adjustment in status open.
pub fn new_retention_adjustment(code: &str, title: &str) -> RetentionAdjustment {
    RetentionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        score: 0.0,
        cluster: "".to_string(),
        reviewer: "".to_string(),
        status: RetentionAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_retention_adjustment("code-956", "title-82");
        assert_eq!(item.status, RetentionAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_retention_adjustment("code-956", "title-82");
        if item.move_to(RetentionAdjustmentStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionAdjustmentStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_retention_adjustment("code-956", "title-82");
        item.status = RetentionAdjustmentStatus::Archived;
        assert!(
            item.move_to(RetentionAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
