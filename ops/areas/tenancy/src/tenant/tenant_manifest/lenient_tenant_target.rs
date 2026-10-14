use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient tenant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTenantTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientTenantTargetStatus {
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

impl std::fmt::Display for LenientTenantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient tenant target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantTarget {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub external_ref: String,
    pub segment: String,
    pub cluster: String,
    pub fingerprint: String,
    pub status: LenientTenantTargetStatus,
}

impl LenientTenantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTenantTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTenantTargetStatus) -> bool {
        match self.status {
            LenientTenantTargetStatus::Open => {
                return next == LenientTenantTargetStatus::InReview;
            }
            LenientTenantTargetStatus::InReview => {
                return next == LenientTenantTargetStatus::Resolved || next == LenientTenantTargetStatus::Reopened;
            }
            LenientTenantTargetStatus::Resolved => {
                return next == LenientTenantTargetStatus::Reopened || next == LenientTenantTargetStatus::Archived;
            }
            LenientTenantTargetStatus::Reopened => {
                return next == LenientTenantTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTenantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient tenant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> LenientTenantTarget {
        LenientTenantTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            segment: self.segment.clone(),
            cluster: self.cluster.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient tenant target in status open.
pub fn new_lenient_tenant_target(code: &str, name: &str) -> LenientTenantTarget {
    LenientTenantTarget {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        grace_days: 0,
        external_ref: "".to_string(),
        segment: "".to_string(),
        cluster: "".to_string(),
        fingerprint: "".to_string(),
        status: LenientTenantTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_tenant_target("code-320", "name-909");
        assert_eq!(item.status, LenientTenantTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_tenant_target("code-320", "name-909");
        if item.move_to(LenientTenantTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTenantTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_tenant_target("code-320", "name-909");
        item.status = LenientTenantTargetStatus::Archived;
        assert!(
            item.move_to(LenientTenantTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
