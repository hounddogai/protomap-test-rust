use super::*;

use acme_kit::text;

/// Where a policy item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyItemStatus {
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

impl std::fmt::Display for PolicyItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy item as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyItem {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub margin: f64,
    pub tags: Vec<String>,
    pub visible: bool,
    pub label: String,
    pub updated_at: i64,
    pub segment: String,
    pub status: PolicyItemStatus,
}

impl PolicyItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyItemStatus) -> bool {
        match self.status {
            PolicyItemStatus::Planned => {
                return next == PolicyItemStatus::InProgress || next == PolicyItemStatus::Cancelled;
            }
            PolicyItemStatus::Scheduled => {
                return next == PolicyItemStatus::Cancelled;
            }
            PolicyItemStatus::InProgress => {
                return next == PolicyItemStatus::Completed || next == PolicyItemStatus::Cancelled;
            }
            PolicyItemStatus::Completed => {
                match next {
                    PolicyItemStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PolicyItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy item", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyItem {
        PolicyItem {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            margin: self.margin,
            tags: self.tags.clone(),
            visible: self.visible,
            label: self.label.clone(),
            updated_at: self.updated_at,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy item in status planned.
pub fn new_policy_item(code: &str, name: &str) -> PolicyItem {
    PolicyItem {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        margin: 0.0,
        tags: Vec::new(),
        visible: false,
        label: "".to_string(),
        updated_at: 0,
        segment: "".to_string(),
        status: PolicyItemStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_item("code-530", "name-804");
        assert_eq!(item.status, PolicyItemStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_policy_item("code-530", "name-804");
        if item.move_to(PolicyItemStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyItemStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_item("code-530", "name-804");
        item.status = PolicyItemStatus::Cancelled;
        assert!(
            item.move_to(PolicyItemStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
