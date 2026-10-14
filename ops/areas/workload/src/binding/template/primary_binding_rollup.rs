use super::*;

use acme_kit::text;

/// Where a primary binding rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBindingRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryBindingRollupStatus {
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

impl std::fmt::Display for PrimaryBindingRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary binding rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRollup {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub source: String,
    pub label: String,
    pub budget: f64,
    pub host: String,
    pub audience: String,
    pub visible: bool,
    pub status: PrimaryBindingRollupStatus,
}

impl PrimaryBindingRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBindingRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBindingRollupStatus) -> bool {
        match self.status {
            PrimaryBindingRollupStatus::Open => {
                match next {
                    PrimaryBindingRollupStatus::InReview | PrimaryBindingRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingRollupStatus::InReview => {
                return next == PrimaryBindingRollupStatus::Reopened;
            }
            PrimaryBindingRollupStatus::Resolved => {
                match next {
                    PrimaryBindingRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingRollupStatus::Reopened => {
                return next == PrimaryBindingRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBindingRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary binding rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.budget * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBindingRollup {
        PrimaryBindingRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            source: self.source.clone(),
            label: self.label.clone(),
            budget: self.budget,
            host: self.host.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary binding rollup in status open.
pub fn new_primary_binding_rollup(code: &str, title: &str) -> PrimaryBindingRollup {
    PrimaryBindingRollup {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        source: "".to_string(),
        label: "".to_string(),
        budget: 0.0,
        host: "".to_string(),
        audience: "".to_string(),
        visible: false,
        status: PrimaryBindingRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_binding_rollup("code-162", "title-874");
        assert_eq!(item.status, PrimaryBindingRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_primary_binding_rollup("code-162", "title-874");
        if item.move_to(PrimaryBindingRollupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryBindingRollupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_binding_rollup("code-162", "title-874");
        item.status = PrimaryBindingRollupStatus::Archived;
        assert!(
            item.move_to(PrimaryBindingRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
