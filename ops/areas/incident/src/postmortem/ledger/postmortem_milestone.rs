use super::*;

use acme_kit::text;

/// Where a postmortem milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemMilestoneStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PostmortemMilestoneStatus {
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

impl std::fmt::Display for PostmortemMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemMilestone {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub locale: String,
    pub region: String,
    pub segment: String,
    pub name: String,
    pub namespace: String,
    pub source: String,
    pub quantity: i64,
    pub status: PostmortemMilestoneStatus,
}

impl PostmortemMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemMilestoneStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemMilestoneStatus) -> bool {
        match self.status {
            PostmortemMilestoneStatus::Planned => {
                match next {
                    PostmortemMilestoneStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemMilestoneStatus::Scheduled => {
                return next == PostmortemMilestoneStatus::Completed || next == PostmortemMilestoneStatus::Cancelled;
            }
            PostmortemMilestoneStatus::InProgress => {
                return next == PostmortemMilestoneStatus::Completed || next == PostmortemMilestoneStatus::Cancelled;
            }
            PostmortemMilestoneStatus::Completed => {
                match next {
                    PostmortemMilestoneStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PostmortemMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem milestone {} cannot move from {} to {}",
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

    pub fn copy(&self) -> PostmortemMilestone {
        PostmortemMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a postmortem milestone in status planned.
pub fn new_postmortem_milestone(code: &str, title: &str) -> PostmortemMilestone {
    PostmortemMilestone {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        segment: "".to_string(),
        name: "".to_string(),
        namespace: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        status: PostmortemMilestoneStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_postmortem_milestone("code-491", "title-573");
        assert_eq!(item.status, PostmortemMilestoneStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_postmortem_milestone("code-491", "title-573");
        if item.move_to(PostmortemMilestoneStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemMilestoneStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_postmortem_milestone("code-491", "title-573");
        item.status = PostmortemMilestoneStatus::Cancelled;
        assert!(
            item.move_to(PostmortemMilestoneStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
