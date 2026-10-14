use super::*;

use acme_kit::text;

/// Where a default redaction snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRedactionSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultRedactionSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default redaction snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshot {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub grace_days: i64,
    pub summary: String,
    pub port: i64,
    pub namespace: String,
    pub segment: String,
    pub tags: Vec<String>,
    pub scope: String,
    pub ratio: f64,
    pub status: DefaultRedactionSnapshotStatus,
}

impl DefaultRedactionSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionSnapshotStatus) -> bool {
        match self.status {
            DefaultRedactionSnapshotStatus::Open => {
                return next == DefaultRedactionSnapshotStatus::Reopened;
            }
            DefaultRedactionSnapshotStatus::InReview => {
                return next == DefaultRedactionSnapshotStatus::Archived;
            }
            DefaultRedactionSnapshotStatus::Resolved => {
                match next {
                    DefaultRedactionSnapshotStatus::Reopened | DefaultRedactionSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionSnapshotStatus::Reopened => {
                match next {
                    DefaultRedactionSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRedactionSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRedactionSnapshot {
        DefaultRedactionSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            grace_days: self.grace_days,
            summary: self.summary.clone(),
            port: self.port,
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            tags: self.tags.clone(),
            scope: self.scope.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default redaction snapshot in status open.
pub fn new_default_redaction_snapshot(code: &str, name: &str) -> DefaultRedactionSnapshot {
    DefaultRedactionSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        grace_days: 0,
        summary: "".to_string(),
        port: 0,
        namespace: "".to_string(),
        segment: "".to_string(),
        tags: Vec::new(),
        scope: "".to_string(),
        ratio: 0.0,
        status: DefaultRedactionSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_redaction_snapshot("code-954", "name-251");
        assert_eq!(item.status, DefaultRedactionSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_default_redaction_snapshot("code-954", "name-251");
        if item.move_to(DefaultRedactionSnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRedactionSnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_redaction_snapshot("code-954", "name-251");
        item.status = DefaultRedactionSnapshotStatus::Archived;
        assert!(
            item.move_to(DefaultRedactionSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
