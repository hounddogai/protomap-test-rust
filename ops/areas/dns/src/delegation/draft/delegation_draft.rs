use super::*;

use acme_kit::text;

/// Where a delegation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DelegationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DelegationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDraft {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub maximum: f64,
    pub quantity: i64,
    pub segment: String,
    pub updated_at: i64,
    pub tenant_id: String,
    pub category: String,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub status: DelegationDraftStatus,
}

impl DelegationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationDraftStatus) -> bool {
        match self.status {
            DelegationDraftStatus::Proposed => {
                match next {
                    DelegationDraftStatus::Approved | DelegationDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationDraftStatus::Approved => {
                return next == DelegationDraftStatus::Rejected;
            }
            DelegationDraftStatus::Rejected => {
                match next {
                    DelegationDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DelegationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation draft {} cannot move from {} to {}",
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
        format!("{} {}", "delegation draft", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationDraft {
        DelegationDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            maximum: self.maximum,
            quantity: self.quantity,
            segment: self.segment.clone(),
            updated_at: self.updated_at,
            tenant_id: self.tenant_id.clone(),
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a delegation draft in status proposed.
pub fn new_delegation_draft(code: &str, name: &str) -> DelegationDraft {
    DelegationDraft {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        maximum: 0.0,
        quantity: 0,
        segment: "".to_string(),
        updated_at: 0,
        tenant_id: "".to_string(),
        category: "".to_string(),
        ttl_seconds: 0,
        margin: 0.0,
        status: DelegationDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_delegation_draft("code-922", "name-789");
        assert_eq!(item.status, DelegationDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_delegation_draft("code-922", "name-789");
        if item.move_to(DelegationDraftStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationDraftStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_delegation_draft("code-922", "name-789");
        item.status = DelegationDraftStatus::Withdrawn;
        assert!(
            item.move_to(DelegationDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
