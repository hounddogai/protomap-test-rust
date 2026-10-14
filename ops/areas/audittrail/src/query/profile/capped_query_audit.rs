use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped query audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQueryAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedQueryAuditStatus {
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

impl std::fmt::Display for CappedQueryAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped query audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAudit {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub quantity: i64,
    pub pinned: bool,
    pub sequence: i64,
    pub reviewer: String,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub status: CappedQueryAuditStatus,
}

impl CappedQueryAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQueryAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQueryAuditStatus) -> bool {
        match self.status {
            CappedQueryAuditStatus::Open => {
                match next {
                    CappedQueryAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryAuditStatus::InReview => {
                return next == CappedQueryAuditStatus::Reopened;
            }
            CappedQueryAuditStatus::Resolved => {
                return next == CappedQueryAuditStatus::Reopened || next == CappedQueryAuditStatus::Archived;
            }
            CappedQueryAuditStatus::Reopened => {
                match next {
                    CappedQueryAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedQueryAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped query audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedQueryAudit {
        CappedQueryAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            quantity: self.quantity,
            pinned: self.pinned,
            sequence: self.sequence,
            reviewer: self.reviewer.clone(),
            budget: self.budget,
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped query audit in status open.
pub fn new_capped_query_audit(code: &str, name: &str) -> CappedQueryAudit {
    CappedQueryAudit {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        quantity: 0,
        pinned: false,
        sequence: 0,
        reviewer: "".to_string(),
        budget: 0.0,
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        status: CappedQueryAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_query_audit("code-202", "name-596");
        assert_eq!(item.status, CappedQueryAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_capped_query_audit("code-202", "name-596");
        if item.move_to(CappedQueryAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedQueryAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_query_audit("code-202", "name-596");
        item.status = CappedQueryAuditStatus::Archived;
        assert!(
            item.move_to(CappedQueryAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
