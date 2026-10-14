use super::*;

use acme_kit::text;

/// Where a delegation record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DelegationRecordStatus {
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

impl std::fmt::Display for DelegationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRecord {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub issuer: String,
    pub port: i64,
    pub description: String,
    pub threshold: f64,
    pub unit: String,
    pub maximum: f64,
    pub status: DelegationRecordStatus,
}

impl DelegationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationRecordStatus) -> bool {
        match self.status {
            DelegationRecordStatus::Open => {
                match next {
                    DelegationRecordStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRecordStatus::InReview => {
                return next == DelegationRecordStatus::Archived;
            }
            DelegationRecordStatus::Resolved => {
                match next {
                    DelegationRecordStatus::Reopened | DelegationRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRecordStatus::Reopened => {
                match next {
                    DelegationRecordStatus::Archived => {
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

    pub fn move_to(&mut self, next: DelegationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation record", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.port as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationRecord {
        DelegationRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            port: self.port,
            description: self.description.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a delegation record in status open.
pub fn new_delegation_record(code: &str, name: &str) -> DelegationRecord {
    DelegationRecord {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        issuer: "".to_string(),
        port: 0,
        description: "".to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        maximum: 0.0,
        status: DelegationRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_delegation_record("code-150", "name-860");
        assert_eq!(item.status, DelegationRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_delegation_record("code-150", "name-860");
        if item.move_to(DelegationRecordStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationRecordStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_delegation_record("code-150", "name-860");
        item.status = DelegationRecordStatus::Archived;
        assert!(
            item.move_to(DelegationRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
