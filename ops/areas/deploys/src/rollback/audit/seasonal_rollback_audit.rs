use super::*;

use acme_kit::text;

/// Where a seasonal rollback audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRollbackAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalRollbackAuditStatus {
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

impl std::fmt::Display for SeasonalRollbackAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal rollback audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackAudit {
    pub code: String,
    pub name: String,
    pub label: String,
    pub cluster: String,
    pub region: String,
    pub cost: f64,
    pub segment: String,
    pub scope: String,
    pub source: String,
    pub status: SeasonalRollbackAuditStatus,
}

impl SeasonalRollbackAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRollbackAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRollbackAuditStatus) -> bool {
        match self.status {
            SeasonalRollbackAuditStatus::Open => {
                match next {
                    SeasonalRollbackAuditStatus::InReview | SeasonalRollbackAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRollbackAuditStatus::InReview => {
                return next == SeasonalRollbackAuditStatus::Reopened;
            }
            SeasonalRollbackAuditStatus::Resolved => {
                return next == SeasonalRollbackAuditStatus::Reopened || next == SeasonalRollbackAuditStatus::Archived;
            }
            SeasonalRollbackAuditStatus::Reopened => {
                match next {
                    SeasonalRollbackAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalRollbackAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollback audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> SeasonalRollbackAudit {
        SeasonalRollbackAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            cost: self.cost,
            segment: self.segment.clone(),
            scope: self.scope.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal rollback audit in status open.
pub fn new_seasonal_rollback_audit(code: &str, name: &str) -> SeasonalRollbackAudit {
    SeasonalRollbackAudit {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        region: "".to_string(),
        cost: 0.0,
        segment: "".to_string(),
        scope: "".to_string(),
        source: "".to_string(),
        status: SeasonalRollbackAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_rollback_audit("code-647", "name-559");
        assert_eq!(item.status, SeasonalRollbackAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_seasonal_rollback_audit("code-647", "name-559");
        if item.move_to(SeasonalRollbackAuditStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRollbackAuditStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_rollback_audit("code-647", "name-559");
        item.status = SeasonalRollbackAuditStatus::Archived;
        assert!(
            item.move_to(SeasonalRollbackAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
