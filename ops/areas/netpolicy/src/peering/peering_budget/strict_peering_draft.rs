use super::*;

use acme_kit::text;

/// Where a strict peering draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPeeringDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictPeeringDraftStatus {
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

impl std::fmt::Display for StrictPeeringDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict peering draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraft {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub timezone: String,
    pub cipher: String,
    pub scope: String,
    pub weight: f64,
    pub source: String,
    pub rotation_days: i64,
    pub capacity: i64,
    pub status: StrictPeeringDraftStatus,
}

impl StrictPeeringDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPeeringDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPeeringDraftStatus) -> bool {
        match self.status {
            StrictPeeringDraftStatus::New => {
                return next == StrictPeeringDraftStatus::Queued || next == StrictPeeringDraftStatus::Done;
            }
            StrictPeeringDraftStatus::Queued => {
                match next {
                    StrictPeeringDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringDraftStatus::Running => {
                match next {
                    StrictPeeringDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPeeringDraftStatus::Done => {
                return next == StrictPeeringDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPeeringDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict peering draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict peering draft", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rotation_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPeeringDraft {
        StrictPeeringDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            scope: self.scope.clone(),
            weight: self.weight,
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict peering draft in status new.
pub fn new_strict_peering_draft(code: &str, name: &str) -> StrictPeeringDraft {
    StrictPeeringDraft {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        timezone: "".to_string(),
        cipher: "".to_string(),
        scope: "".to_string(),
        weight: 0.0,
        source: "".to_string(),
        rotation_days: 0,
        capacity: 0,
        status: StrictPeeringDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_peering_draft("code-900", "name-942");
        assert_eq!(item.status, StrictPeeringDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_peering_draft("code-900", "name-942");
        if item.move_to(StrictPeeringDraftStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPeeringDraftStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_peering_draft("code-900", "name-942");
        item.status = StrictPeeringDraftStatus::Failed;
        assert!(
            item.move_to(StrictPeeringDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
