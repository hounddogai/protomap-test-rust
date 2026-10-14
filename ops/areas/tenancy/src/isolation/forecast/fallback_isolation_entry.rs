use super::*;

use acme_kit::text;

/// Where a fallback isolation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIsolationEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackIsolationEntryStatus {
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

impl std::fmt::Display for FallbackIsolationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback isolation entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntry {
    pub code: String,
    pub name: String,
    pub country: String,
    pub capacity: i64,
    pub key_id: String,
    pub sequence: i64,
    pub ceiling: f64,
    pub notes: String,
    pub budget: f64,
    pub status: FallbackIsolationEntryStatus,
}

impl FallbackIsolationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIsolationEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIsolationEntryStatus) -> bool {
        match self.status {
            FallbackIsolationEntryStatus::New => {
                match next {
                    FallbackIsolationEntryStatus::Queued | FallbackIsolationEntryStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIsolationEntryStatus::Queued => {
                return next == FallbackIsolationEntryStatus::Failed;
            }
            FallbackIsolationEntryStatus::Running => {
                return next == FallbackIsolationEntryStatus::Done || next == FallbackIsolationEntryStatus::Failed;
            }
            FallbackIsolationEntryStatus::Done => {
                match next {
                    FallbackIsolationEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackIsolationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback isolation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIsolationEntry {
        FallbackIsolationEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            ceiling: self.ceiling,
            notes: self.notes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback isolation entry in status new.
pub fn new_fallback_isolation_entry(code: &str, name: &str) -> FallbackIsolationEntry {
    FallbackIsolationEntry {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        capacity: 0,
        key_id: "".to_string(),
        sequence: 0,
        ceiling: 0.0,
        notes: "".to_string(),
        budget: 0.0,
        status: FallbackIsolationEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_isolation_entry("code-153", "name-639");
        assert_eq!(item.status, FallbackIsolationEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_isolation_entry("code-153", "name-639");
        if item.move_to(FallbackIsolationEntryStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackIsolationEntryStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_isolation_entry("code-153", "name-639");
        item.status = FallbackIsolationEntryStatus::Failed;
        assert!(
            item.move_to(FallbackIsolationEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
