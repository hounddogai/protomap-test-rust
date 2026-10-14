use super::*;

use acme_kit::text;

/// Where a binding template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BindingTemplateStatus {
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

impl std::fmt::Display for BindingTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTemplate {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub segment: String,
    pub active: bool,
    pub quantity: i64,
    pub status: BindingTemplateStatus,
}

impl BindingTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingTemplateStatus) -> bool {
        match self.status {
            BindingTemplateStatus::Planned => {
                match next {
                    BindingTemplateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingTemplateStatus::Scheduled => {
                match next {
                    BindingTemplateStatus::InProgress | BindingTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingTemplateStatus::InProgress => {
                match next {
                    BindingTemplateStatus::Completed | BindingTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingTemplateStatus::Completed => {
                return next == BindingTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> BindingTemplate {
        BindingTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            segment: self.segment.clone(),
            active: self.active,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a binding template in status planned.
pub fn new_binding_template(code: &str, title: &str) -> BindingTemplate {
    BindingTemplate {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        segment: "".to_string(),
        active: false,
        quantity: 0,
        status: BindingTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_binding_template("code-18", "title-533");
        assert_eq!(item.status, BindingTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_binding_template("code-18", "title-533");
        if item.move_to(BindingTemplateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingTemplateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_binding_template("code-18", "title-533");
        item.status = BindingTemplateStatus::Cancelled;
        assert!(
            item.move_to(BindingTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
