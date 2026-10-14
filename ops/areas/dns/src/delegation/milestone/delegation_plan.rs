use super::*;

use acme_kit::text;

/// Where a delegation plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation plan as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlan {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub pinned: bool,
    pub priority: i64,
    pub version: i64,
    pub threshold: f64,
    pub label: String,
    pub limit: i64,
    pub status: DelegationPlanStatus,
}

impl DelegationPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationPlanStatus) -> bool {
        match self.status {
            DelegationPlanStatus::Created => {
                match next {
                    DelegationPlanStatus::Validated | DelegationPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationPlanStatus::Validated => {
                return next == DelegationPlanStatus::Published;
            }
            DelegationPlanStatus::Published => {
                match next {
                    DelegationPlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: DelegationPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.priority as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationPlan {
        DelegationPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            pinned: self.pinned,
            priority: self.priority,
            version: self.version,
            threshold: self.threshold,
            label: self.label.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a delegation plan in status created.
pub fn new_delegation_plan(code: &str, title: &str) -> DelegationPlan {
    DelegationPlan {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        pinned: false,
        priority: 0,
        version: 0,
        threshold: 0.0,
        label: "".to_string(),
        limit: 0,
        status: DelegationPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_delegation_plan("code-227", "title-600");
        assert_eq!(item.status, DelegationPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_delegation_plan("code-227", "title-600");
        if item.move_to(DelegationPlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationPlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_delegation_plan("code-227", "title-600");
        item.status = DelegationPlanStatus::Retired;
        assert!(
            item.move_to(DelegationPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
