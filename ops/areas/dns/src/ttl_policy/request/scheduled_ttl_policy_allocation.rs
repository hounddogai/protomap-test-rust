use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled ttl policy allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTtlPolicyAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledTtlPolicyAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledTtlPolicyAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled ttl policy allocation as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocation {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub tags: Vec<String>,
    pub active: bool,
    pub pinned: bool,
    pub updated_at: i64,
    pub locale: String,
    pub phone: String,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub status: ScheduledTtlPolicyAllocationStatus,
}

impl ScheduledTtlPolicyAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTtlPolicyAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTtlPolicyAllocationStatus) -> bool {
        match self.status {
            ScheduledTtlPolicyAllocationStatus::Created => {
                match next {
                    ScheduledTtlPolicyAllocationStatus::Published | ScheduledTtlPolicyAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTtlPolicyAllocationStatus::Validated => {
                return next == ScheduledTtlPolicyAllocationStatus::Retired;
            }
            ScheduledTtlPolicyAllocationStatus::Published => {
                match next {
                    ScheduledTtlPolicyAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledTtlPolicyAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled ttl policy allocation {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled ttl policy allocation", self.code)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.updated_at as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTtlPolicyAllocation {
        ScheduledTtlPolicyAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            tags: self.tags.clone(),
            active: self.active,
            pinned: self.pinned,
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled ttl policy allocation in status created.
pub fn new_scheduled_ttl_policy_allocation(code: &str, label: &str) -> ScheduledTtlPolicyAllocation {
    ScheduledTtlPolicyAllocation {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        tags: Vec::new(),
        active: false,
        pinned: false,
        updated_at: 0,
        locale: "".to_string(),
        phone: "".to_string(),
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        status: ScheduledTtlPolicyAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_ttl_policy_allocation("code-647", "label-739");
        assert_eq!(item.status, ScheduledTtlPolicyAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_ttl_policy_allocation("code-647", "label-739");
        if item.move_to(ScheduledTtlPolicyAllocationStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTtlPolicyAllocationStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_ttl_policy_allocation("code-647", "label-739");
        item.status = ScheduledTtlPolicyAllocationStatus::Retired;
        assert!(
            item.move_to(ScheduledTtlPolicyAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
