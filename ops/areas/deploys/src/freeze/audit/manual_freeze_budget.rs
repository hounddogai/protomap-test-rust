use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual freeze budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFreezeBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualFreezeBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualFreezeBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual freeze budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFreezeBudget {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub principal: String,
    pub budget: f64,
    pub cluster: String,
    pub retries: i64,
    pub priority: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: ManualFreezeBudgetStatus,
}

impl ManualFreezeBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFreezeBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFreezeBudgetStatus) -> bool {
        match self.status {
            ManualFreezeBudgetStatus::Proposed => {
                return next == ManualFreezeBudgetStatus::Withdrawn;
            }
            ManualFreezeBudgetStatus::Approved => {
                match next {
                    ManualFreezeBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFreezeBudgetStatus::Rejected => {
                match next {
                    ManualFreezeBudgetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualFreezeBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual freeze budget {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
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

    pub fn projected_total(&self) -> f64 {
        (self.budget * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFreezeBudget {
        ManualFreezeBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            principal: self.principal.clone(),
            budget: self.budget,
            cluster: self.cluster.clone(),
            retries: self.retries,
            priority: self.priority,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual freeze budget in status proposed.
pub fn new_manual_freeze_budget(code: &str, name: &str) -> ManualFreezeBudget {
    ManualFreezeBudget {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        principal: "".to_string(),
        budget: 0.0,
        cluster: "".to_string(),
        retries: 0,
        priority: 0,
        attributes: BTreeMap::new(),
        status: ManualFreezeBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_freeze_budget("code-643", "name-937");
        assert_eq!(item.status, ManualFreezeBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_freeze_budget("code-643", "name-937");
        if item.move_to(ManualFreezeBudgetStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFreezeBudgetStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_freeze_budget("code-643", "name-937");
        item.status = ManualFreezeBudgetStatus::Withdrawn;
        assert!(
            item.move_to(ManualFreezeBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
