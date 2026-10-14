use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PlanAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAdjustment {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub port: i64,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub priority: i64,
    pub contact_email: String,
    pub description: String,
    pub amount: f64,
    pub status: PlanAdjustmentStatus,
}

impl PlanAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanAdjustmentStatus) -> bool {
        match self.status {
            PlanAdjustmentStatus::Planned => {
                match next {
                    PlanAdjustmentStatus::Scheduled | PlanAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanAdjustmentStatus::Scheduled => {
                return next == PlanAdjustmentStatus::Cancelled;
            }
            PlanAdjustmentStatus::InProgress => {
                return next == PlanAdjustmentStatus::Completed || next == PlanAdjustmentStatus::Cancelled;
            }
            PlanAdjustmentStatus::Completed => {
                match next {
                    PlanAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PlanAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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

    pub fn net_value(&self) -> f64 {
        ((self.priority as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanAdjustment {
        PlanAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            port: self.port,
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            visible: self.visible,
            priority: self.priority,
            contact_email: self.contact_email.clone(),
            description: self.description.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan adjustment in status planned.
pub fn new_plan_adjustment(code: &str, title: &str) -> PlanAdjustment {
    PlanAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        port: 0,
        host: "".to_string(),
        attributes: BTreeMap::new(),
        visible: false,
        priority: 0,
        contact_email: "".to_string(),
        description: "".to_string(),
        amount: 0.0,
        status: PlanAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_adjustment("code-702", "title-6");
        assert_eq!(item.status, PlanAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_plan_adjustment("code-702", "title-6");
        if item.move_to(PlanAdjustmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanAdjustmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_adjustment("code-702", "title-6");
        item.status = PlanAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(PlanAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
