use super::*;

use acme_kit::text;

/// Where a global release adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalReleaseAdjustmentStatus {
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

impl std::fmt::Display for GlobalReleaseAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global release adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustment {
    pub code: String,
    pub title: String,
    pub name: String,
    pub quantity: i64,
    pub sequence: i64,
    pub namespace: String,
    pub quota_units: i64,
    pub window_days: i64,
    pub priority: i64,
    pub limit: i64,
    pub amount: f64,
    pub status: GlobalReleaseAdjustmentStatus,
}

impl GlobalReleaseAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseAdjustmentStatus) -> bool {
        match self.status {
            GlobalReleaseAdjustmentStatus::Open => {
                match next {
                    GlobalReleaseAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAdjustmentStatus::InReview => {
                match next {
                    GlobalReleaseAdjustmentStatus::Reopened | GlobalReleaseAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAdjustmentStatus::Resolved => {
                match next {
                    GlobalReleaseAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAdjustmentStatus::Reopened => {
                return next == GlobalReleaseAdjustmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReleaseAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.window_days as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleaseAdjustment {
        GlobalReleaseAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            sequence: self.sequence,
            namespace: self.namespace.clone(),
            quota_units: self.quota_units,
            window_days: self.window_days,
            priority: self.priority,
            limit: self.limit,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global release adjustment in status open.
pub fn new_global_release_adjustment(code: &str, title: &str) -> GlobalReleaseAdjustment {
    GlobalReleaseAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        quantity: 0,
        sequence: 0,
        namespace: "".to_string(),
        quota_units: 0,
        window_days: 0,
        priority: 0,
        limit: 0,
        amount: 0.0,
        status: GlobalReleaseAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_release_adjustment("code-454", "title-319");
        assert_eq!(item.status, GlobalReleaseAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_global_release_adjustment("code-454", "title-319");
        if item.move_to(GlobalReleaseAdjustmentStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReleaseAdjustmentStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_release_adjustment("code-454", "title-319");
        item.status = GlobalReleaseAdjustmentStatus::Archived;
        assert!(
            item.move_to(GlobalReleaseAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
