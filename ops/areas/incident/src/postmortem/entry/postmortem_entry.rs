use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PostmortemEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PostmortemEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem entry as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemEntry {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub category: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub status: PostmortemEntryStatus,
}

impl PostmortemEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemEntryStatus) -> bool {
        match self.status {
            PostmortemEntryStatus::Proposed => {
                match next {
                    PostmortemEntryStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemEntryStatus::Approved => {
                return next == PostmortemEntryStatus::Rejected || next == PostmortemEntryStatus::Withdrawn;
            }
            PostmortemEntryStatus::Rejected => {
                return next == PostmortemEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem entry", self.code)
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
        self.cost
    }

    pub fn copy(&self) -> PostmortemEntry {
        PostmortemEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            category: self.category.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a postmortem entry in status proposed.
pub fn new_postmortem_entry(code: &str, label: &str) -> PostmortemEntry {
    PostmortemEntry {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        category: "".to_string(),
        title: "".to_string(),
        attributes: BTreeMap::new(),
        status: PostmortemEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_postmortem_entry("code-349", "label-844");
        assert_eq!(item.status, PostmortemEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_postmortem_entry("code-349", "label-844");
        if item.move_to(PostmortemEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_postmortem_entry("code-349", "label-844");
        item.status = PostmortemEntryStatus::Withdrawn;
        assert!(
            item.move_to(PostmortemEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
