use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PostmortemManifestStatus {
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

impl std::fmt::Display for PostmortemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem manifest as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemManifest {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub owner_id: String,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: PostmortemManifestStatus,
}

impl PostmortemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemManifestStatus) -> bool {
        match self.status {
            PostmortemManifestStatus::Planned => {
                return next == PostmortemManifestStatus::Scheduled || next == PostmortemManifestStatus::Cancelled;
            }
            PostmortemManifestStatus::Scheduled => {
                match next {
                    PostmortemManifestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemManifestStatus::InProgress => {
                match next {
                    PostmortemManifestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemManifestStatus::Completed => {
                return next == PostmortemManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        self.minimum
    }

    pub fn copy(&self) -> PostmortemManifest {
        PostmortemManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem manifest in status planned.
pub fn new_postmortem_manifest(code: &str, title: &str) -> PostmortemManifest {
    PostmortemManifest {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        owner_id: "".to_string(),
        minimum: 0.0,
        attributes: BTreeMap::new(),
        status: PostmortemManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_postmortem_manifest("code-120", "title-583");
        assert_eq!(item.status, PostmortemManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_postmortem_manifest("code-120", "title-583");
        if item.move_to(PostmortemManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_postmortem_manifest("code-120", "title-583");
        item.status = PostmortemManifestStatus::Cancelled;
        assert!(
            item.move_to(PostmortemManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
