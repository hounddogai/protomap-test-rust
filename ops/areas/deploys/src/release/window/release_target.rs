use super::*;

use acme_kit::text;

/// Where a release target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseTargetStatus {
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

impl std::fmt::Display for ReleaseTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTarget {
    pub code: String,
    pub name: String,
    pub region: String,
    pub active: bool,
    pub maximum: f64,
    pub pinned: bool,
    pub retries: i64,
    pub currency: String,
    pub locale: String,
    pub status: ReleaseTargetStatus,
}

impl ReleaseTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseTargetStatus) -> bool {
        match self.status {
            ReleaseTargetStatus::Open => {
                match next {
                    ReleaseTargetStatus::Resolved | ReleaseTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseTargetStatus::InReview => {
                return next == ReleaseTargetStatus::Resolved || next == ReleaseTargetStatus::Archived;
            }
            ReleaseTargetStatus::Resolved => {
                match next {
                    ReleaseTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseTargetStatus::Reopened => {
                return next == ReleaseTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        (self.maximum * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseTarget {
        ReleaseTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            active: self.active,
            maximum: self.maximum,
            pinned: self.pinned,
            retries: self.retries,
            currency: self.currency.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a release target in status open.
pub fn new_release_target(code: &str, name: &str) -> ReleaseTarget {
    ReleaseTarget {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        active: false,
        maximum: 0.0,
        pinned: false,
        retries: 0,
        currency: "".to_string(),
        locale: "".to_string(),
        status: ReleaseTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_target("code-595", "name-349");
        assert_eq!(item.status, ReleaseTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_release_target("code-595", "name-349");
        if item.move_to(ReleaseTargetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseTargetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_target("code-595", "name-349");
        item.status = ReleaseTargetStatus::Archived;
        assert!(
            item.move_to(ReleaseTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
