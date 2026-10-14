use super::*;

use acme_kit::text;

/// Where a scheduled identity bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIdentityBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledIdentityBundleStatus {
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

impl std::fmt::Display for ScheduledIdentityBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled identity bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityBundle {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub priority: i64,
    pub amount: f64,
    pub unit: String,
    pub label: String,
    pub status: ScheduledIdentityBundleStatus,
}

impl ScheduledIdentityBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIdentityBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIdentityBundleStatus) -> bool {
        match self.status {
            ScheduledIdentityBundleStatus::Planned => {
                match next {
                    ScheduledIdentityBundleStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityBundleStatus::Scheduled => {
                return next == ScheduledIdentityBundleStatus::InProgress
                    || next == ScheduledIdentityBundleStatus::Cancelled;
            }
            ScheduledIdentityBundleStatus::InProgress => {
                match next {
                    ScheduledIdentityBundleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIdentityBundleStatus::Completed => {
                return next == ScheduledIdentityBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledIdentityBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled identity bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.amount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledIdentityBundle {
        ScheduledIdentityBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            amount: self.amount,
            unit: self.unit.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a scheduled identity bundle in status planned.
pub fn new_scheduled_identity_bundle(code: &str, name: &str) -> ScheduledIdentityBundle {
    ScheduledIdentityBundle {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        priority: 0,
        amount: 0.0,
        unit: "".to_string(),
        label: "".to_string(),
        status: ScheduledIdentityBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_identity_bundle("code-587", "name-378");
        assert_eq!(item.status, ScheduledIdentityBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_scheduled_identity_bundle("code-587", "name-378");
        if item.move_to(ScheduledIdentityBundleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledIdentityBundleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_identity_bundle("code-587", "name-378");
        item.status = ScheduledIdentityBundleStatus::Cancelled;
        assert!(
            item.move_to(ScheduledIdentityBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
