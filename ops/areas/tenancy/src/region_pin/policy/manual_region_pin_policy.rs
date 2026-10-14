use super::*;

use acme_kit::text;

/// Where a manual region pin policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRegionPinPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualRegionPinPolicyStatus {
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

impl std::fmt::Display for ManualRegionPinPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual region pin policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicy {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub category: String,
    pub summary: String,
    pub name: String,
    pub archived: bool,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub updated_at: i64,
    pub status: ManualRegionPinPolicyStatus,
}

impl ManualRegionPinPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRegionPinPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRegionPinPolicyStatus) -> bool {
        match self.status {
            ManualRegionPinPolicyStatus::Open => {
                return next == ManualRegionPinPolicyStatus::InReview;
            }
            ManualRegionPinPolicyStatus::InReview => {
                return next == ManualRegionPinPolicyStatus::Reopened;
            }
            ManualRegionPinPolicyStatus::Resolved => {
                return next == ManualRegionPinPolicyStatus::Reopened || next == ManualRegionPinPolicyStatus::Archived;
            }
            ManualRegionPinPolicyStatus::Reopened => {
                return next == ManualRegionPinPolicyStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRegionPinPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual region pin policy {} cannot move from {} to {}",
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
        format!("{} {}", "manual region pin policy", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRegionPinPolicy {
        ManualRegionPinPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            category: self.category.clone(),
            summary: self.summary.clone(),
            name: self.name.clone(),
            archived: self.archived,
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual region pin policy in status open.
pub fn new_manual_region_pin_policy(code: &str, title: &str) -> ManualRegionPinPolicy {
    ManualRegionPinPolicy {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        category: "".to_string(),
        summary: "".to_string(),
        name: "".to_string(),
        archived: false,
        contact_email: "".to_string(),
        ttl_seconds: 0,
        minimum: 0.0,
        updated_at: 0,
        status: ManualRegionPinPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_region_pin_policy("code-143", "title-29");
        assert_eq!(item.status, ManualRegionPinPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_region_pin_policy("code-143", "title-29");
        if item.move_to(ManualRegionPinPolicyStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRegionPinPolicyStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_region_pin_policy("code-143", "title-29");
        item.status = ManualRegionPinPolicyStatus::Archived;
        assert!(
            item.move_to(ManualRegionPinPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
