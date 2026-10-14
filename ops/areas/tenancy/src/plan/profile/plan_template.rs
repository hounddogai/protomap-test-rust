use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTemplate {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub version: i64,
    pub ratio: f64,
    pub channel: String,
    pub scope: String,
    pub grace_days: i64,
    pub status: PlanTemplateStatus,
}

impl PlanTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanTemplateStatus) -> bool {
        match self.status {
            PlanTemplateStatus::Proposed => {
                return next == PlanTemplateStatus::Approved || next == PlanTemplateStatus::Rejected;
            }
            PlanTemplateStatus::Approved => {
                match next {
                    PlanTemplateStatus::Rejected | PlanTemplateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanTemplateStatus::Rejected => {
                match next {
                    PlanTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PlanTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn projected_value(&self) -> f64 {
        ((self.version as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanTemplate {
        PlanTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            discount: self.discount,
            version: self.version,
            ratio: self.ratio,
            channel: self.channel.clone(),
            scope: self.scope.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a plan template in status proposed.
pub fn new_plan_template(code: &str, label: &str) -> PlanTemplate {
    PlanTemplate {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        discount: 0.0,
        version: 0,
        ratio: 0.0,
        channel: "".to_string(),
        scope: "".to_string(),
        grace_days: 0,
        status: PlanTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_template("code-281", "label-104");
        assert_eq!(item.status, PlanTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_plan_template("code-281", "label-104");
        if item.move_to(PlanTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_template("code-281", "label-104");
        item.status = PlanTemplateStatus::Withdrawn;
        assert!(
            item.move_to(PlanTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
