use super::*;

use acme_kit::text;

/// Where a release template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseTemplateStatus {
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

impl std::fmt::Display for ReleaseTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplate {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub audience: String,
    pub visible: bool,
    pub grace_days: i64,
    pub cluster: String,
    pub archived: bool,
    pub quota_units: i64,
    pub status: ReleaseTemplateStatus,
}

impl ReleaseTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseTemplateStatus) -> bool {
        match self.status {
            ReleaseTemplateStatus::Open => {
                return next == ReleaseTemplateStatus::Resolved || next == ReleaseTemplateStatus::Archived;
            }
            ReleaseTemplateStatus::InReview => {
                match next {
                    ReleaseTemplateStatus::Resolved | ReleaseTemplateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseTemplateStatus::Resolved => {
                return next == ReleaseTemplateStatus::Reopened || next == ReleaseTemplateStatus::Archived;
            }
            ReleaseTemplateStatus::Reopened => {
                match next {
                    ReleaseTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReleaseTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "release template", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseTemplate {
        ReleaseTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            visible: self.visible,
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            archived: self.archived,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a release template in status open.
pub fn new_release_template(code: &str, label: &str) -> ReleaseTemplate {
    ReleaseTemplate {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        ttl_seconds: 0,
        audience: "".to_string(),
        visible: false,
        grace_days: 0,
        cluster: "".to_string(),
        archived: false,
        quota_units: 0,
        status: ReleaseTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_template("code-401", "label-34");
        assert_eq!(item.status, ReleaseTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_release_template("code-401", "label-34");
        if item.move_to(ReleaseTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_template("code-401", "label-34");
        item.status = ReleaseTemplateStatus::Archived;
        assert!(
            item.move_to(ReleaseTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
