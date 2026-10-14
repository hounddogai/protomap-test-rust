use super::*;

use acme_kit::text;

/// Where a rolling silence group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSilenceGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingSilenceGroupStatus {
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

impl std::fmt::Display for RollingSilenceGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling silence group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceGroup {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub minimum: f64,
    pub batch_size: i64,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub namespace: String,
    pub status: RollingSilenceGroupStatus,
}

impl RollingSilenceGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSilenceGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSilenceGroupStatus) -> bool {
        match self.status {
            RollingSilenceGroupStatus::New => {
                return next == RollingSilenceGroupStatus::Queued || next == RollingSilenceGroupStatus::Failed;
            }
            RollingSilenceGroupStatus::Queued => {
                match next {
                    RollingSilenceGroupStatus::Done | RollingSilenceGroupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSilenceGroupStatus::Running => {
                return next == RollingSilenceGroupStatus::Failed;
            }
            RollingSilenceGroupStatus::Done => {
                match next {
                    RollingSilenceGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingSilenceGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling silence group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSilenceGroup {
        RollingSilenceGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            minimum: self.minimum,
            batch_size: self.batch_size,
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling silence group in status new.
pub fn new_rolling_silence_group(code: &str, name: &str) -> RollingSilenceGroup {
    RollingSilenceGroup {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        minimum: 0.0,
        batch_size: 0,
        revision: 0,
        ttl_seconds: 0,
        namespace: "".to_string(),
        status: RollingSilenceGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_silence_group("code-642", "name-551");
        assert_eq!(item.status, RollingSilenceGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rolling_silence_group("code-642", "name-551");
        if item.move_to(RollingSilenceGroupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSilenceGroupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_silence_group("code-642", "name-551");
        item.status = RollingSilenceGroupStatus::Failed;
        assert!(
            item.move_to(RollingSilenceGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
