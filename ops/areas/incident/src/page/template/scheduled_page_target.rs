use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled page target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPageTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledPageTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledPageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled page target as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTarget {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub owner_id: String,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub status: ScheduledPageTargetStatus,
}

impl ScheduledPageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPageTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPageTargetStatus) -> bool {
        match self.status {
            ScheduledPageTargetStatus::Draft => {
                return next == ScheduledPageTargetStatus::Pending || next == ScheduledPageTargetStatus::Suspended;
            }
            ScheduledPageTargetStatus::Pending => {
                return next == ScheduledPageTargetStatus::Active || next == ScheduledPageTargetStatus::Suspended;
            }
            ScheduledPageTargetStatus::Active => {
                return next == ScheduledPageTargetStatus::Suspended || next == ScheduledPageTargetStatus::Closed;
            }
            ScheduledPageTargetStatus::Suspended => {
                match next {
                    ScheduledPageTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledPageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled page target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ScheduledPageTarget {
        ScheduledPageTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled page target in status draft.
pub fn new_scheduled_page_target(code: &str, name: &str) -> ScheduledPageTarget {
    ScheduledPageTarget {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        owner_id: "".to_string(),
        attributes: BTreeMap::new(),
        score: 0.0,
        status: ScheduledPageTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_page_target("code-563", "name-28");
        assert_eq!(item.status, ScheduledPageTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_page_target("code-563", "name-28");
        if item.move_to(ScheduledPageTargetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPageTargetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_page_target("code-563", "name-28");
        item.status = ScheduledPageTargetStatus::Closed;
        assert!(
            item.move_to(ScheduledPageTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
