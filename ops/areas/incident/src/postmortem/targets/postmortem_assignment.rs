use super::*;

use acme_kit::text;

/// Where a postmortem assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PostmortemAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PostmortemAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAssignment {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub tags: Vec<String>,
    pub namespace: String,
    pub batch_size: i64,
    pub revision: i64,
    pub attempts: i64,
    pub label: String,
    pub status: PostmortemAssignmentStatus,
}

impl PostmortemAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemAssignmentStatus) -> bool {
        match self.status {
            PostmortemAssignmentStatus::Created => {
                return next == PostmortemAssignmentStatus::Validated || next == PostmortemAssignmentStatus::Retired;
            }
            PostmortemAssignmentStatus::Validated => {
                return next == PostmortemAssignmentStatus::Retired;
            }
            PostmortemAssignmentStatus::Published => {
                match next {
                    PostmortemAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PostmortemAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemAssignment {
        PostmortemAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            batch_size: self.batch_size,
            revision: self.revision,
            attempts: self.attempts,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a postmortem assignment in status created.
pub fn new_postmortem_assignment(code: &str, title: &str) -> PostmortemAssignment {
    PostmortemAssignment {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        tags: Vec::new(),
        namespace: "".to_string(),
        batch_size: 0,
        revision: 0,
        attempts: 0,
        label: "".to_string(),
        status: PostmortemAssignmentStatus::Created,
    }
}
