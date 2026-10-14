use super::*;

use acme_kit::text;

/// Where a slo rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloRollupStatus {
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

impl std::fmt::Display for SloRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloRollup {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub name: String,
    pub ceiling: f64,
    pub archived: bool,
    pub status: SloRollupStatus,
}

impl SloRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloRollupStatus) -> bool {
        match self.status {
            SloRollupStatus::Open => {
                return next == SloRollupStatus::InReview || next == SloRollupStatus::Resolved;
            }
            SloRollupStatus::InReview => {
                match next {
                    SloRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloRollupStatus::Resolved => {
                return next == SloRollupStatus::Reopened;
            }
            SloRollupStatus::Reopened => {
                match next {
                    SloRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: SloRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.ceiling * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloRollup {
        SloRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            name: self.name.clone(),
            ceiling: self.ceiling,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo rollup in status open.
pub fn new_slo_rollup(code: &str, label: &str) -> SloRollup {
    SloRollup {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        name: "".to_string(),
        ceiling: 0.0,
        archived: false,
        status: SloRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_rollup("code-413", "label-476");
        assert_eq!(item.status, SloRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_slo_rollup("code-413", "label-476");
        if item.move_to(SloRollupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloRollupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_rollup("code-413", "label-476");
        item.status = SloRollupStatus::Archived;
        assert!(item.move_to(SloRollupStatus::Open).is_err(), "expected move to fail");
    }
}
