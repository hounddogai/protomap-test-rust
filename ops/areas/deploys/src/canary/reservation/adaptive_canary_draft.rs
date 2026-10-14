use super::*;

use acme_kit::text;

/// Where a adaptive canary draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCanaryDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveCanaryDraftStatus {
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

impl std::fmt::Display for AdaptiveCanaryDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive canary draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraft {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub threshold: f64,
    pub priority: i64,
    pub status: AdaptiveCanaryDraftStatus,
}

impl AdaptiveCanaryDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCanaryDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCanaryDraftStatus) -> bool {
        match self.status {
            AdaptiveCanaryDraftStatus::New => {
                return next == AdaptiveCanaryDraftStatus::Done || next == AdaptiveCanaryDraftStatus::Failed;
            }
            AdaptiveCanaryDraftStatus::Queued => {
                match next {
                    AdaptiveCanaryDraftStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryDraftStatus::Running => {
                match next {
                    AdaptiveCanaryDraftStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveCanaryDraftStatus::Done => {
                return next == AdaptiveCanaryDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveCanaryDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive canary draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveCanaryDraft {
        AdaptiveCanaryDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive canary draft in status new.
pub fn new_adaptive_canary_draft(code: &str, title: &str) -> AdaptiveCanaryDraft {
    AdaptiveCanaryDraft {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        threshold: 0.0,
        priority: 0,
        status: AdaptiveCanaryDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_canary_draft("code-320", "title-161");
        assert_eq!(item.status, AdaptiveCanaryDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_adaptive_canary_draft("code-320", "title-161");
        if item.move_to(AdaptiveCanaryDraftStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveCanaryDraftStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_canary_draft("code-320", "title-161");
        item.status = AdaptiveCanaryDraftStatus::Failed;
        assert!(
            item.move_to(AdaptiveCanaryDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
