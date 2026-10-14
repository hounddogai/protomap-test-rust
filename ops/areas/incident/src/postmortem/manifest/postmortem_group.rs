use super::*;

use acme_kit::text;

/// Where a postmortem group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PostmortemGroupStatus {
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

impl std::fmt::Display for PostmortemGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroup {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub cipher: String,
    pub namespace: String,
    pub summary: String,
    pub retries: i64,
    pub margin: f64,
    pub status: PostmortemGroupStatus,
}

impl PostmortemGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemGroupStatus) -> bool {
        match self.status {
            PostmortemGroupStatus::Planned => {
                return next == PostmortemGroupStatus::Scheduled;
            }
            PostmortemGroupStatus::Scheduled => {
                return next == PostmortemGroupStatus::InProgress || next == PostmortemGroupStatus::Cancelled;
            }
            PostmortemGroupStatus::InProgress => {
                return next == PostmortemGroupStatus::Completed;
            }
            PostmortemGroupStatus::Completed => {
                return next == PostmortemGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemGroup {
        PostmortemGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            cipher: self.cipher.clone(),
            namespace: self.namespace.clone(),
            summary: self.summary.clone(),
            retries: self.retries,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a postmortem group in status planned.
pub fn new_postmortem_group(code: &str, name: &str) -> PostmortemGroup {
    PostmortemGroup {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        cipher: "".to_string(),
        namespace: "".to_string(),
        summary: "".to_string(),
        retries: 0,
        margin: 0.0,
        status: PostmortemGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_postmortem_group("code-791", "name-379");
        assert_eq!(item.status, PostmortemGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_postmortem_group("code-791", "name-379");
        if item.move_to(PostmortemGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_postmortem_group("code-791", "name-379");
        item.status = PostmortemGroupStatus::Cancelled;
        assert!(
            item.move_to(PostmortemGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
