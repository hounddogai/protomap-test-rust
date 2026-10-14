use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a suspension plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionPlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SuspensionPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SuspensionPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionPlan {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub unit: String,
    pub key_id: String,
    pub external_ref: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub description: String,
    pub status: SuspensionPlanStatus,
}

impl SuspensionPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionPlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionPlanStatus) -> bool {
        match self.status {
            SuspensionPlanStatus::Proposed => {
                match next {
                    SuspensionPlanStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionPlanStatus::Approved => {
                return next == SuspensionPlanStatus::Withdrawn;
            }
            SuspensionPlanStatus::Rejected => {
                match next {
                    SuspensionPlanStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SuspensionPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> SuspensionPlan {
        SuspensionPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a suspension plan in status proposed.
pub fn new_suspension_plan(code: &str, title: &str) -> SuspensionPlan {
    SuspensionPlan {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        unit: "".to_string(),
        key_id: "".to_string(),
        external_ref: "".to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        description: "".to_string(),
        status: SuspensionPlanStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_suspension_plan("code-782", "title-37");
        assert_eq!(item.status, SuspensionPlanStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_suspension_plan("code-782", "title-37");
        if item.move_to(SuspensionPlanStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionPlanStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_suspension_plan("code-782", "title-37");
        item.status = SuspensionPlanStatus::Withdrawn;
        assert!(
            item.move_to(SuspensionPlanStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
