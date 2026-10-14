use super::*;

use acme_kit::text;

/// Where a plan quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanQuota {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub port: i64,
    pub maximum: f64,
    pub category: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub capacity: i64,
    pub quantity: i64,
    pub status: PlanQuotaStatus,
}

impl PlanQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanQuotaStatus) -> bool {
        match self.status {
            PlanQuotaStatus::Proposed => {
                match next {
                    PlanQuotaStatus::Approved | PlanQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanQuotaStatus::Approved => {
                return next == PlanQuotaStatus::Rejected || next == PlanQuotaStatus::Withdrawn;
            }
            PlanQuotaStatus::Rejected => {
                match next {
                    PlanQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PlanQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanQuota {
        PlanQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            port: self.port,
            maximum: self.maximum,
            category: self.category.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a plan quota in status proposed.
pub fn new_plan_quota(code: &str, name: &str) -> PlanQuota {
    PlanQuota {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        port: 0,
        maximum: 0.0,
        category: "".to_string(),
        priority: 0,
        ttl_seconds: 0,
        tags: Vec::new(),
        capacity: 0,
        quantity: 0,
        status: PlanQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_quota("code-57", "name-876");
        assert_eq!(item.status, PlanQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_plan_quota("code-57", "name-876");
        if item.move_to(PlanQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_quota("code-57", "name-876");
        item.status = PlanQuotaStatus::Withdrawn;
        assert!(
            item.move_to(PlanQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
