use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched tenant milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTenantMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedTenantMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedTenantMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched tenant milestone as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantMilestone {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub margin: f64,
    pub scope: String,
    pub audience: String,
    pub rotation_days: i64,
    pub principal: String,
    pub category: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub status: BatchedTenantMilestoneStatus,
}

impl BatchedTenantMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTenantMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTenantMilestoneStatus) -> bool {
        match self.status {
            BatchedTenantMilestoneStatus::Created => {
                match next {
                    BatchedTenantMilestoneStatus::Published | BatchedTenantMilestoneStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTenantMilestoneStatus::Validated => {
                return next == BatchedTenantMilestoneStatus::Published;
            }
            BatchedTenantMilestoneStatus::Published => {
                return next == BatchedTenantMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTenantMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched tenant milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn net_load(&self) -> f64 {
        ((self.rotation_days as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTenantMilestone {
        BatchedTenantMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            margin: self.margin,
            scope: self.scope.clone(),
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            principal: self.principal.clone(),
            category: self.category.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched tenant milestone in status created.
pub fn new_batched_tenant_milestone(code: &str, label: &str) -> BatchedTenantMilestone {
    BatchedTenantMilestone {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        margin: 0.0,
        scope: "".to_string(),
        audience: "".to_string(),
        rotation_days: 0,
        principal: "".to_string(),
        category: "".to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        status: BatchedTenantMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_tenant_milestone("code-921", "label-630");
        assert_eq!(item.status, BatchedTenantMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_batched_tenant_milestone("code-921", "label-630");
        if item.move_to(BatchedTenantMilestoneStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTenantMilestoneStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_tenant_milestone("code-921", "label-630");
        item.status = BatchedTenantMilestoneStatus::Retired;
        assert!(
            item.move_to(BatchedTenantMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
