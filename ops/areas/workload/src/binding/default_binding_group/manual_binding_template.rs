use super::*;

use acme_kit::text;

/// Where a manual binding template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBindingTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualBindingTemplateStatus {
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

impl std::fmt::Display for ManualBindingTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual binding template as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTemplate {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub sequence: i64,
    pub floor_value: f64,
    pub attempts: i64,
    pub status: ManualBindingTemplateStatus,
}

impl ManualBindingTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBindingTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBindingTemplateStatus) -> bool {
        match self.status {
            ManualBindingTemplateStatus::Planned => {
                match next {
                    ManualBindingTemplateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBindingTemplateStatus::Scheduled => {
                return next == ManualBindingTemplateStatus::InProgress
                    || next == ManualBindingTemplateStatus::Completed;
            }
            ManualBindingTemplateStatus::InProgress => {
                return next == ManualBindingTemplateStatus::Completed
                    || next == ManualBindingTemplateStatus::Cancelled;
            }
            ManualBindingTemplateStatus::Completed => {
                match next {
                    ManualBindingTemplateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualBindingTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual binding template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.floor_value - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBindingTemplate {
        ManualBindingTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            sequence: self.sequence,
            floor_value: self.floor_value,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual binding template in status planned.
pub fn new_manual_binding_template(code: &str, label: &str) -> ManualBindingTemplate {
    ManualBindingTemplate {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        sequence: 0,
        floor_value: 0.0,
        attempts: 0,
        status: ManualBindingTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_binding_template("code-124", "label-306");
        assert_eq!(item.status, ManualBindingTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_manual_binding_template("code-124", "label-306");
        if item.move_to(ManualBindingTemplateStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBindingTemplateStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_binding_template("code-124", "label-306");
        item.status = ManualBindingTemplateStatus::Cancelled;
        assert!(
            item.move_to(ManualBindingTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
