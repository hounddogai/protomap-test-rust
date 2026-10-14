use super::*;

use acme_kit::text;

/// Where a fallback postmortem threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackPostmortemThresholdStatus {
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

impl std::fmt::Display for FallbackPostmortemThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback postmortem threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemThreshold {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub cluster: String,
    pub created_at: i64,
    pub audience: String,
    pub visible: bool,
    pub batch_size: i64,
    pub amount: f64,
    pub status: FallbackPostmortemThresholdStatus,
}

impl FallbackPostmortemThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemThresholdStatus) -> bool {
        match self.status {
            FallbackPostmortemThresholdStatus::Open => {
                match next {
                    FallbackPostmortemThresholdStatus::Reopened | FallbackPostmortemThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemThresholdStatus::InReview => {
                return next == FallbackPostmortemThresholdStatus::Resolved;
            }
            FallbackPostmortemThresholdStatus::Resolved => {
                match next {
                    FallbackPostmortemThresholdStatus::Reopened | FallbackPostmortemThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemThresholdStatus::Reopened => {
                return next == FallbackPostmortemThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPostmortemThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem threshold {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        (self.amount * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemThreshold {
        FallbackPostmortemThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            cluster: self.cluster.clone(),
            created_at: self.created_at,
            audience: self.audience.clone(),
            visible: self.visible,
            batch_size: self.batch_size,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback postmortem threshold in status open.
pub fn new_fallback_postmortem_threshold(code: &str, title: &str) -> FallbackPostmortemThreshold {
    FallbackPostmortemThreshold {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        cluster: "".to_string(),
        created_at: 0,
        audience: "".to_string(),
        visible: false,
        batch_size: 0,
        amount: 0.0,
        status: FallbackPostmortemThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_postmortem_threshold("code-842", "title-816");
        assert_eq!(item.status, FallbackPostmortemThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_fallback_postmortem_threshold("code-842", "title-816");
        if item.move_to(FallbackPostmortemThresholdStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemThresholdStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_postmortem_threshold("code-842", "title-816");
        item.status = FallbackPostmortemThresholdStatus::Archived;
        assert!(
            item.move_to(FallbackPostmortemThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
