use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending federation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingFederationNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingFederationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending federation notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationNotice {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub visible: bool,
    pub attributes: BTreeMap<String, String>,
    pub owner_id: String,
    pub limit: i64,
    pub status: PendingFederationNoticeStatus,
}

impl PendingFederationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationNoticeStatus) -> bool {
        match self.status {
            PendingFederationNoticeStatus::Proposed => {
                match next {
                    PendingFederationNoticeStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationNoticeStatus::Approved => {
                return next == PendingFederationNoticeStatus::Withdrawn;
            }
            PendingFederationNoticeStatus::Rejected => {
                return next == PendingFederationNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingFederationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn effective_load(&self) -> f64 {
        ((self.limit as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationNotice {
        PendingFederationNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            visible: self.visible,
            attributes: self.attributes.clone(),
            owner_id: self.owner_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending federation notice in status proposed.
pub fn new_pending_federation_notice(code: &str, title: &str) -> PendingFederationNotice {
    PendingFederationNotice {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        visible: false,
        attributes: BTreeMap::new(),
        owner_id: "".to_string(),
        limit: 0,
        status: PendingFederationNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_federation_notice("code-753", "title-880");
        assert_eq!(item.status, PendingFederationNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_federation_notice("code-753", "title-880");
        if item.move_to(PendingFederationNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFederationNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_federation_notice("code-753", "title-880");
        item.status = PendingFederationNoticeStatus::Withdrawn;
        assert!(
            item.move_to(PendingFederationNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
