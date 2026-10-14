use super::*;

use acme_kit::text;

/// Where a probe plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbePlanStatus {
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

impl std::fmt::Display for ProbePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePlan {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub segment: String,
    pub minimum: f64,
    pub version: i64,
    pub summary: String,
    pub archived: bool,
    pub status: ProbePlanStatus,
}

impl ProbePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbePlanStatus) -> bool {
        match self.status {
            ProbePlanStatus::Open => {
                return next == ProbePlanStatus::InReview || next == ProbePlanStatus::Archived;
            }
            ProbePlanStatus::InReview => {
                match next {
                    ProbePlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePlanStatus::Resolved => {
                match next {
                    ProbePlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePlanStatus::Reopened => {
                match next {
                    ProbePlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: ProbePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbePlan {
        ProbePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            minimum: self.minimum,
            version: self.version,
            summary: self.summary.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a probe plan in status open.
pub fn new_probe_plan(code: &str, label: &str) -> ProbePlan {
    ProbePlan {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        segment: "".to_string(),
        minimum: 0.0,
        version: 0,
        summary: "".to_string(),
        archived: false,
        status: ProbePlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_plan("code-151", "label-351");
        assert_eq!(item.status, ProbePlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_probe_plan("code-151", "label-351");
        if item.move_to(ProbePlanStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbePlanStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_plan("code-151", "label-351");
        item.status = ProbePlanStatus::Archived;
        assert!(item.move_to(ProbePlanStatus::Open).is_err(), "expected move to fail");
    }
}
