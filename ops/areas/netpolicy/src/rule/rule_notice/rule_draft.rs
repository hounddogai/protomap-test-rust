use super::*;

use acme_kit::text;

/// Where a rule draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleDraftStatus {
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

impl std::fmt::Display for RuleDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDraft {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub notes: String,
    pub version: i64,
    pub pinned: bool,
    pub threshold: f64,
    pub segment: String,
    pub namespace: String,
    pub status: RuleDraftStatus,
}

impl RuleDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleDraftStatus) -> bool {
        match self.status {
            RuleDraftStatus::Open => {
                match next {
                    RuleDraftStatus::Reopened | RuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleDraftStatus::InReview => {
                match next {
                    RuleDraftStatus::Resolved | RuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleDraftStatus::Resolved => {
                match next {
                    RuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleDraftStatus::Reopened => {
                return next == RuleDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleDraft {
        RuleDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            version: self.version,
            pinned: self.pinned,
            threshold: self.threshold,
            segment: self.segment.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rule draft in status open.
pub fn new_rule_draft(code: &str, name: &str) -> RuleDraft {
    RuleDraft {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        notes: "".to_string(),
        version: 0,
        pinned: false,
        threshold: 0.0,
        segment: "".to_string(),
        namespace: "".to_string(),
        status: RuleDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rule_draft("code-871", "name-706");
        assert_eq!(item.status, RuleDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_rule_draft("code-871", "name-706");
        if item.move_to(RuleDraftStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleDraftStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rule_draft("code-871", "name-706");
        item.status = RuleDraftStatus::Archived;
        assert!(item.move_to(RuleDraftStatus::Open).is_err(), "expected move to fail");
    }
}
