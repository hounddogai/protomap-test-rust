use super::*;

use acme_kit::text;

/// Where a rollback bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollbackBundleStatus {
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

impl std::fmt::Display for RollbackBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBundle {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub channel: String,
    pub summary: String,
    pub source: String,
    pub contact_email: String,
    pub updated_at: i64,
    pub ends_at: i64,
    pub status: RollbackBundleStatus,
}

impl RollbackBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackBundleStatus) -> bool {
        match self.status {
            RollbackBundleStatus::Open => {
                return next == RollbackBundleStatus::Resolved;
            }
            RollbackBundleStatus::InReview => {
                return next == RollbackBundleStatus::Reopened;
            }
            RollbackBundleStatus::Resolved => {
                match next {
                    RollbackBundleStatus::Reopened | RollbackBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackBundleStatus::Reopened => {
                match next {
                    RollbackBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollbackBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ends_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackBundle {
        RollbackBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            channel: self.channel.clone(),
            summary: self.summary.clone(),
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rollback bundle in status open.
pub fn new_rollback_bundle(code: &str, name: &str) -> RollbackBundle {
    RollbackBundle {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        channel: "".to_string(),
        summary: "".to_string(),
        source: "".to_string(),
        contact_email: "".to_string(),
        updated_at: 0,
        ends_at: 0,
        status: RollbackBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rollback_bundle("code-189", "name-326");
        assert_eq!(item.status, RollbackBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rollback_bundle("code-189", "name-326");
        if item.move_to(RollbackBundleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackBundleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rollback_bundle("code-189", "name-326");
        item.status = RollbackBundleStatus::Archived;
        assert!(
            item.move_to(RollbackBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
