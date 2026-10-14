use super::*;

use acme_kit::text;

/// Where a action item template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemTemplateStatus {
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

impl std::fmt::Display for ActionItemTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplate {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub pinned: bool,
    pub external_ref: String,
    pub owner_id: String,
    pub discount: f64,
    pub limit: i64,
    pub scope: String,
    pub ratio: f64,
    pub status: ActionItemTemplateStatus,
}

impl ActionItemTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemTemplateStatus) -> bool {
        match self.status {
            ActionItemTemplateStatus::Planned => {
                match next {
                    ActionItemTemplateStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemTemplateStatus::Scheduled => {
                return next == ActionItemTemplateStatus::Cancelled;
            }
            ActionItemTemplateStatus::InProgress => {
                return next == ActionItemTemplateStatus::Cancelled;
            }
            ActionItemTemplateStatus::Completed => {
                return next == ActionItemTemplateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.limit as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemTemplate {
        ActionItemTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            pinned: self.pinned,
            external_ref: self.external_ref.clone(),
            owner_id: self.owner_id.clone(),
            discount: self.discount,
            limit: self.limit,
            scope: self.scope.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a action item template in status planned.
pub fn new_action_item_template(code: &str, title: &str) -> ActionItemTemplate {
    ActionItemTemplate {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        pinned: false,
        external_ref: "".to_string(),
        owner_id: "".to_string(),
        discount: 0.0,
        limit: 0,
        scope: "".to_string(),
        ratio: 0.0,
        status: ActionItemTemplateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_action_item_template("code-767", "title-701");
        assert_eq!(item.status, ActionItemTemplateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_action_item_template("code-767", "title-701");
        if item.move_to(ActionItemTemplateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemTemplateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_action_item_template("code-767", "title-701");
        item.status = ActionItemTemplateStatus::Cancelled;
        assert!(
            item.move_to(ActionItemTemplateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
