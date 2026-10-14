use super::*;

use acme_kit::text;

/// Where a allowlist review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AllowlistReviewStatus {
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

impl std::fmt::Display for AllowlistReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist review as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReview {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub checksum: String,
    pub country: String,
    pub phone: String,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub rate: f64,
    pub status: AllowlistReviewStatus,
}

impl AllowlistReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistReviewStatus) -> bool {
        match self.status {
            AllowlistReviewStatus::Open => {
                return next == AllowlistReviewStatus::Resolved;
            }
            AllowlistReviewStatus::InReview => {
                match next {
                    AllowlistReviewStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistReviewStatus::Resolved => {
                return next == AllowlistReviewStatus::Reopened || next == AllowlistReviewStatus::Archived;
            }
            AllowlistReviewStatus::Reopened => {
                return next == AllowlistReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist review {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist review", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistReview {
        AllowlistReview {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            country: self.country.clone(),
            phone: self.phone.clone(),
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a allowlist review in status open.
pub fn new_allowlist_review(code: &str, name: &str) -> AllowlistReview {
    AllowlistReview {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        checksum: "".to_string(),
        country: "".to_string(),
        phone: "".to_string(),
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        rate: 0.0,
        status: AllowlistReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_allowlist_review("code-378", "name-851");
        assert_eq!(item.status, AllowlistReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_allowlist_review("code-378", "name-851");
        if item.move_to(AllowlistReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_allowlist_review("code-378", "name-851");
        item.status = AllowlistReviewStatus::Archived;
        assert!(
            item.move_to(AllowlistReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
