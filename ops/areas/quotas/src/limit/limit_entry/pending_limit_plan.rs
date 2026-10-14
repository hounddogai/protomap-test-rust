use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending limit plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingLimitPlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingLimitPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingLimitPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending limit plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlan {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub attributes: BTreeMap<String, String>,
    pub quota_units: i64,
    pub ends_at: i64,
    pub tenant_id: String,
    pub priority: i64,
    pub description: String,
    pub scope: String,
    pub updated_at: i64,
    pub status: PendingLimitPlanStatus,
}

impl PendingLimitPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingLimitPlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingLimitPlanStatus) -> bool {
        match self.status {
            PendingLimitPlanStatus::Proposed => {
                match next {
                    PendingLimitPlanStatus::Approved | PendingLimitPlanStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingLimitPlanStatus::Approved => {
                return next == PendingLimitPlanStatus::Rejected;
            }
            PendingLimitPlanStatus::Rejected => {
                match next {
                    PendingLimitPlanStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingLimitPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending limit plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingLimitPlan {
        PendingLimitPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            attributes: self.attributes.clone(),
            quota_units: self.quota_units,
            ends_at: self.ends_at,
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            description: self.description.clone(),
            scope: self.scope.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending limit plan in status proposed.
pub fn new_pending_limit_plan(code: &str, title: &str) -> PendingLimitPlan {
    PendingLimitPlan {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        attributes: BTreeMap::new(),
        quota_units: 0,
        ends_at: 0,
        tenant_id: "".to_string(),
        priority: 0,
        description: "".to_string(),
        scope: "".to_string(),
        updated_at: 0,
        status: PendingLimitPlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_limit_plan("code-753", "title-125");
        assert_eq!(item.status, PendingLimitPlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_limit_plan("code-753", "title-125");
        if item.move_to(PendingLimitPlanStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingLimitPlanStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_limit_plan("code-753", "title-125");
        item.status = PendingLimitPlanStatus::Withdrawn;
        assert!(
            item.move_to(PendingLimitPlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
