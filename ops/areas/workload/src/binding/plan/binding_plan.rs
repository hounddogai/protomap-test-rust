use super::*;

use acme_kit::text;

/// Where a binding plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BindingPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BindingPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding plan as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlan {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub cost: f64,
    pub batch_size: i64,
    pub ratio: f64,
    pub status: BindingPlanStatus,
}

impl BindingPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingPlanStatus) -> bool {
        match self.status {
            BindingPlanStatus::Created => {
                match next {
                    BindingPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingPlanStatus::Validated => {
                return next == BindingPlanStatus::Published || next == BindingPlanStatus::Retired;
            }
            BindingPlanStatus::Published => {
                match next {
                    BindingPlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: BindingPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingPlan {
        BindingPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            cost: self.cost,
            batch_size: self.batch_size,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding plan in status created.
pub fn new_binding_plan(code: &str, name: &str) -> BindingPlan {
    BindingPlan {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        cost: 0.0,
        batch_size: 0,
        ratio: 0.0,
        status: BindingPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_binding_plan("code-269", "name-172");
        assert_eq!(item.status, BindingPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_binding_plan("code-269", "name-172");
        if item.move_to(BindingPlanStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingPlanStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_binding_plan("code-269", "name-172");
        item.status = BindingPlanStatus::Retired;
        assert!(
            item.move_to(BindingPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
