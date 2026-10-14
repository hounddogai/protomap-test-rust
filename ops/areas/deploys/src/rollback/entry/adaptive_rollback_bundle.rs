use super::*;

use acme_kit::text;

/// Where a adaptive rollback bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRollbackBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveRollbackBundleStatus {
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

impl std::fmt::Display for AdaptiveRollbackBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollback bundle as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackBundle {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub label: String,
    pub namespace: String,
    pub priority: i64,
    pub description: String,
    pub audience: String,
    pub ratio: f64,
    pub notes: String,
    pub starts_at: i64,
    pub status: AdaptiveRollbackBundleStatus,
}

impl AdaptiveRollbackBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRollbackBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRollbackBundleStatus) -> bool {
        match self.status {
            AdaptiveRollbackBundleStatus::Open => {
                match next {
                    AdaptiveRollbackBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRollbackBundleStatus::InReview => {
                return next == AdaptiveRollbackBundleStatus::Reopened;
            }
            AdaptiveRollbackBundleStatus::Resolved => {
                match next {
                    AdaptiveRollbackBundleStatus::Reopened | AdaptiveRollbackBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRollbackBundleStatus::Reopened => {
                match next {
                    AdaptiveRollbackBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveRollbackBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollback bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.starts_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRollbackBundle {
        AdaptiveRollbackBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            priority: self.priority,
            description: self.description.clone(),
            audience: self.audience.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive rollback bundle in status open.
pub fn new_adaptive_rollback_bundle(code: &str, title: &str) -> AdaptiveRollbackBundle {
    AdaptiveRollbackBundle {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        label: "".to_string(),
        namespace: "".to_string(),
        priority: 0,
        description: "".to_string(),
        audience: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        starts_at: 0,
        status: AdaptiveRollbackBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_rollback_bundle("code-151", "title-365");
        assert_eq!(item.status, AdaptiveRollbackBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_adaptive_rollback_bundle("code-151", "title-365");
        if item.move_to(AdaptiveRollbackBundleStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRollbackBundleStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_rollback_bundle("code-151", "title-365");
        item.status = AdaptiveRollbackBundleStatus::Archived;
        assert!(
            item.move_to(AdaptiveRollbackBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
