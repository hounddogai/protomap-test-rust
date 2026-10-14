use super::*;

use acme_kit::text;

/// Where a adaptive probe draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveProbeDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveProbeDraftStatus {
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

impl std::fmt::Display for AdaptiveProbeDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive probe draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeDraft {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub amount: f64,
    pub score: f64,
    pub issuer: String,
    pub ends_at: i64,
    pub quota_units: i64,
    pub minimum: f64,
    pub scope: String,
    pub visible: bool,
    pub status: AdaptiveProbeDraftStatus,
}

impl AdaptiveProbeDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveProbeDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveProbeDraftStatus) -> bool {
        match self.status {
            AdaptiveProbeDraftStatus::New => {
                return next == AdaptiveProbeDraftStatus::Running;
            }
            AdaptiveProbeDraftStatus::Queued => {
                match next {
                    AdaptiveProbeDraftStatus::Running | AdaptiveProbeDraftStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeDraftStatus::Running => {
                match next {
                    AdaptiveProbeDraftStatus::Done | AdaptiveProbeDraftStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeDraftStatus::Done => {
                return next == AdaptiveProbeDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveProbeDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive probe draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.sequence as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveProbeDraft {
        AdaptiveProbeDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            amount: self.amount,
            score: self.score,
            issuer: self.issuer.clone(),
            ends_at: self.ends_at,
            quota_units: self.quota_units,
            minimum: self.minimum,
            scope: self.scope.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive probe draft in status new.
pub fn new_adaptive_probe_draft(code: &str, label: &str) -> AdaptiveProbeDraft {
    AdaptiveProbeDraft {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        amount: 0.0,
        score: 0.0,
        issuer: "".to_string(),
        ends_at: 0,
        quota_units: 0,
        minimum: 0.0,
        scope: "".to_string(),
        visible: false,
        status: AdaptiveProbeDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_probe_draft("code-132", "label-698");
        assert_eq!(item.status, AdaptiveProbeDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_probe_draft("code-132", "label-698");
        if item.move_to(AdaptiveProbeDraftStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveProbeDraftStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_probe_draft("code-132", "label-698");
        item.status = AdaptiveProbeDraftStatus::Failed;
        assert!(
            item.move_to(AdaptiveProbeDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
