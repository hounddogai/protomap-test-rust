use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary approval notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryApprovalNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryApprovalNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryApprovalNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary approval notice as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNotice {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub key_id: String,
    pub name: String,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub timezone: String,
    pub batch_size: i64,
    pub status: PrimaryApprovalNoticeStatus,
}

impl PrimaryApprovalNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryApprovalNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryApprovalNoticeStatus) -> bool {
        match self.status {
            PrimaryApprovalNoticeStatus::Requested => {
                match next {
                    PrimaryApprovalNoticeStatus::Granted | PrimaryApprovalNoticeStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryApprovalNoticeStatus::Granted => {
                return next == PrimaryApprovalNoticeStatus::Expired;
            }
            PrimaryApprovalNoticeStatus::Revoked => {
                return next == PrimaryApprovalNoticeStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryApprovalNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary approval notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn net_value(&self) -> f64 {
        ((self.budget - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryApprovalNotice {
        PrimaryApprovalNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            key_id: self.key_id.clone(),
            name: self.name.clone(),
            budget: self.budget,
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary approval notice in status requested.
pub fn new_primary_approval_notice(code: &str, title: &str) -> PrimaryApprovalNotice {
    PrimaryApprovalNotice {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        key_id: "".to_string(),
        name: "".to_string(),
        budget: 0.0,
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        created_at: 0,
        timezone: "".to_string(),
        batch_size: 0,
        status: PrimaryApprovalNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_approval_notice("code-253", "title-162");
        assert_eq!(item.status, PrimaryApprovalNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_approval_notice("code-253", "title-162");
        if item.move_to(PrimaryApprovalNoticeStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryApprovalNoticeStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_approval_notice("code-253", "title-162");
        item.status = PrimaryApprovalNoticeStatus::Expired;
        assert!(
            item.move_to(PrimaryApprovalNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
