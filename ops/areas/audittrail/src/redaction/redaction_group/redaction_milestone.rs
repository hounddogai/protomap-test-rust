use super::*;

use acme_kit::text;

/// Where a redaction milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RedactionMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RedactionMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestone {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub tags: Vec<String>,
    pub starts_at: i64,
    pub summary: String,
    pub cost: f64,
    pub maximum: f64,
    pub cluster: String,
    pub rotation_days: i64,
    pub status: RedactionMilestoneStatus,
}

impl RedactionMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionMilestoneStatus) -> bool {
        match self.status {
            RedactionMilestoneStatus::Proposed => {
                return next == RedactionMilestoneStatus::Rejected || next == RedactionMilestoneStatus::Withdrawn;
            }
            RedactionMilestoneStatus::Approved => {
                match next {
                    RedactionMilestoneStatus::Rejected | RedactionMilestoneStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionMilestoneStatus::Rejected => {
                match next {
                    RedactionMilestoneStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RedactionMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionMilestone {
        RedactionMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            tags: self.tags.clone(),
            starts_at: self.starts_at,
            summary: self.summary.clone(),
            cost: self.cost,
            maximum: self.maximum,
            cluster: self.cluster.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a redaction milestone in status proposed.
pub fn new_redaction_milestone(code: &str, name: &str) -> RedactionMilestone {
    RedactionMilestone {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        tags: Vec::new(),
        starts_at: 0,
        summary: "".to_string(),
        cost: 0.0,
        maximum: 0.0,
        cluster: "".to_string(),
        rotation_days: 0,
        status: RedactionMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_redaction_milestone("code-329", "name-800");
        assert_eq!(item.status, RedactionMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_redaction_milestone("code-329", "name-800");
        if item.move_to(RedactionMilestoneStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionMilestoneStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_redaction_milestone("code-329", "name-800");
        item.status = RedactionMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(RedactionMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
