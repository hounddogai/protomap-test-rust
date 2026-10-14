use super::*;

use acme_kit::text;

/// Where a identity milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IdentityMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for IdentityMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One identity milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestone {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub cluster: String,
    pub channel: String,
    pub external_ref: String,
    pub maximum: f64,
    pub timezone: String,
    pub status: IdentityMilestoneStatus,
}

impl IdentityMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityMilestoneStatus) -> bool {
        match self.status {
            IdentityMilestoneStatus::New => {
                return next == IdentityMilestoneStatus::Done || next == IdentityMilestoneStatus::Failed;
            }
            IdentityMilestoneStatus::Queued => {
                match next {
                    IdentityMilestoneStatus::Running | IdentityMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityMilestoneStatus::Running => {
                return next == IdentityMilestoneStatus::Failed;
            }
            IdentityMilestoneStatus::Done => {
                match next {
                    IdentityMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: IdentityMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> IdentityMilestone {
        IdentityMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            channel: self.channel.clone(),
            external_ref: self.external_ref.clone(),
            maximum: self.maximum,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a identity milestone in status new.
pub fn new_identity_milestone(code: &str, label: &str) -> IdentityMilestone {
    IdentityMilestone {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        cluster: "".to_string(),
        channel: "".to_string(),
        external_ref: "".to_string(),
        maximum: 0.0,
        timezone: "".to_string(),
        status: IdentityMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_identity_milestone("code-650", "label-104");
        assert_eq!(item.status, IdentityMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_identity_milestone("code-650", "label-104");
        if item.move_to(IdentityMilestoneStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityMilestoneStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_identity_milestone("code-650", "label-104");
        item.status = IdentityMilestoneStatus::Failed;
        assert!(
            item.move_to(IdentityMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
