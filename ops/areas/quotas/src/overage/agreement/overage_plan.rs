use super::*;

use acme_kit::text;

/// Where a overage plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OveragePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OveragePlanStatus {
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

impl std::fmt::Display for OveragePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage plan as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePlan {
    pub code: String,
    pub title: String,
    pub source: String,
    pub revision: i64,
    pub notes: String,
    pub ratio: f64,
    pub scope: String,
    pub capacity: i64,
    pub checksum: String,
    pub label: String,
    pub status: OveragePlanStatus,
}

impl OveragePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OveragePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OveragePlanStatus) -> bool {
        match self.status {
            OveragePlanStatus::Open => {
                return next == OveragePlanStatus::InReview;
            }
            OveragePlanStatus::InReview => {
                match next {
                    OveragePlanStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OveragePlanStatus::Resolved => {
                match next {
                    OveragePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OveragePlanStatus::Reopened => {
                return next == OveragePlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OveragePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage plan", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.revision as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OveragePlan {
        OveragePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            revision: self.revision,
            notes: self.notes.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            capacity: self.capacity,
            checksum: self.checksum.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a overage plan in status open.
pub fn new_overage_plan(code: &str, title: &str) -> OveragePlan {
    OveragePlan {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        revision: 0,
        notes: "".to_string(),
        ratio: 0.0,
        scope: "".to_string(),
        capacity: 0,
        checksum: "".to_string(),
        label: "".to_string(),
        status: OveragePlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_overage_plan("code-235", "title-951");
        assert_eq!(item.status, OveragePlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_overage_plan("code-235", "title-951");
        if item.move_to(OveragePlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OveragePlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_overage_plan("code-235", "title-951");
        item.status = OveragePlanStatus::Archived;
        assert!(item.move_to(OveragePlanStatus::Open).is_err(), "expected move to fail");
    }
}
