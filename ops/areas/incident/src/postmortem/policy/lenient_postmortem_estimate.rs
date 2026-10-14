use super::*;

use acme_kit::text;

/// Where a lenient postmortem estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPostmortemEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientPostmortemEstimateStatus {
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

impl std::fmt::Display for LenientPostmortemEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient postmortem estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemEstimate {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub checksum: String,
    pub summary: String,
    pub created_at: i64,
    pub status: LenientPostmortemEstimateStatus,
}

impl LenientPostmortemEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPostmortemEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPostmortemEstimateStatus) -> bool {
        match self.status {
            LenientPostmortemEstimateStatus::New => {
                return next == LenientPostmortemEstimateStatus::Running
                    || next == LenientPostmortemEstimateStatus::Failed;
            }
            LenientPostmortemEstimateStatus::Queued => {
                return next == LenientPostmortemEstimateStatus::Done
                    || next == LenientPostmortemEstimateStatus::Failed;
            }
            LenientPostmortemEstimateStatus::Running => {
                return next == LenientPostmortemEstimateStatus::Failed;
            }
            LenientPostmortemEstimateStatus::Done => {
                return next == LenientPostmortemEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPostmortemEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient postmortem estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPostmortemEstimate {
        LenientPostmortemEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient postmortem estimate in status new.
pub fn new_lenient_postmortem_estimate(code: &str, title: &str) -> LenientPostmortemEstimate {
    LenientPostmortemEstimate {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        checksum: "".to_string(),
        summary: "".to_string(),
        created_at: 0,
        status: LenientPostmortemEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_postmortem_estimate("code-907", "title-448");
        assert_eq!(item.status, LenientPostmortemEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_lenient_postmortem_estimate("code-907", "title-448");
        if item.move_to(LenientPostmortemEstimateStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPostmortemEstimateStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_postmortem_estimate("code-907", "title-448");
        item.status = LenientPostmortemEstimateStatus::Failed;
        assert!(
            item.move_to(LenientPostmortemEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
