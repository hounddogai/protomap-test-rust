use super::*;

use acme_kit::text;

/// Where a identity record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IdentityRecordStatus {
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

impl std::fmt::Display for IdentityRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecord {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub category: String,
    pub cipher: String,
    pub region: String,
    pub quota_units: i64,
    pub checksum: String,
    pub port: i64,
    pub approved: bool,
    pub status: IdentityRecordStatus,
}

impl IdentityRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityRecordStatus) -> bool {
        match self.status {
            IdentityRecordStatus::Open => {
                return next == IdentityRecordStatus::InReview || next == IdentityRecordStatus::Resolved;
            }
            IdentityRecordStatus::InReview => {
                match next {
                    IdentityRecordStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityRecordStatus::Resolved => {
                match next {
                    IdentityRecordStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityRecordStatus::Reopened => {
                return next == IdentityRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quota_units as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityRecord {
        IdentityRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            cipher: self.cipher.clone(),
            region: self.region.clone(),
            quota_units: self.quota_units,
            checksum: self.checksum.clone(),
            port: self.port,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a identity record in status open.
pub fn new_identity_record(code: &str, title: &str) -> IdentityRecord {
    IdentityRecord {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        category: "".to_string(),
        cipher: "".to_string(),
        region: "".to_string(),
        quota_units: 0,
        checksum: "".to_string(),
        port: 0,
        approved: false,
        status: IdentityRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_identity_record("code-557", "title-116");
        assert_eq!(item.status, IdentityRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_identity_record("code-557", "title-116");
        if item.move_to(IdentityRecordStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityRecordStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_identity_record("code-557", "title-116");
        item.status = IdentityRecordStatus::Archived;
        assert!(
            item.move_to(IdentityRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
