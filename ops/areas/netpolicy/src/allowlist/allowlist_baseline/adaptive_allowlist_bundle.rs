use super::*;

use acme_kit::text;

/// Where a adaptive allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAllowlistBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveAllowlistBundleStatus {
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

impl std::fmt::Display for AdaptiveAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive allowlist bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBundle {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub description: String,
    pub category: String,
    pub owner_id: String,
    pub status: AdaptiveAllowlistBundleStatus,
}

impl AdaptiveAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAllowlistBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAllowlistBundleStatus) -> bool {
        match self.status {
            AdaptiveAllowlistBundleStatus::Open => {
                match next {
                    AdaptiveAllowlistBundleStatus::InReview | AdaptiveAllowlistBundleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBundleStatus::InReview => {
                match next {
                    AdaptiveAllowlistBundleStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBundleStatus::Resolved => {
                match next {
                    AdaptiveAllowlistBundleStatus::Reopened | AdaptiveAllowlistBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistBundleStatus::Reopened => {
                return next == AdaptiveAllowlistBundleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive allowlist bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive allowlist bundle", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> AdaptiveAllowlistBundle {
        AdaptiveAllowlistBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            description: self.description.clone(),
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive allowlist bundle in status open.
pub fn new_adaptive_allowlist_bundle(code: &str, name: &str) -> AdaptiveAllowlistBundle {
    AdaptiveAllowlistBundle {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        description: "".to_string(),
        category: "".to_string(),
        owner_id: "".to_string(),
        status: AdaptiveAllowlistBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_allowlist_bundle("code-468", "name-629");
        assert_eq!(item.status, AdaptiveAllowlistBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_adaptive_allowlist_bundle("code-468", "name-629");
        if item.move_to(AdaptiveAllowlistBundleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAllowlistBundleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_allowlist_bundle("code-468", "name-629");
        item.status = AdaptiveAllowlistBundleStatus::Archived;
        assert!(
            item.move_to(AdaptiveAllowlistBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
