use super::*;

use acme_kit::text;

/// Where a plan draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanDraftStatus {
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

impl std::fmt::Display for PlanDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDraft {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub fingerprint: String,
    pub score: f64,
    pub currency: String,
    pub host: String,
    pub channel: String,
    pub unit: String,
    pub status: PlanDraftStatus,
}

impl PlanDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanDraftStatus) -> bool {
        match self.status {
            PlanDraftStatus::Open => {
                return next == PlanDraftStatus::Reopened;
            }
            PlanDraftStatus::InReview => {
                match next {
                    PlanDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanDraftStatus::Resolved => {
                match next {
                    PlanDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanDraftStatus::Reopened => {
                match next {
                    PlanDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: PlanDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanDraft {
        PlanDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            currency: self.currency.clone(),
            host: self.host.clone(),
            channel: self.channel.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a plan draft in status open.
pub fn new_plan_draft(code: &str, title: &str) -> PlanDraft {
    PlanDraft {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        fingerprint: "".to_string(),
        score: 0.0,
        currency: "".to_string(),
        host: "".to_string(),
        channel: "".to_string(),
        unit: "".to_string(),
        status: PlanDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_plan_draft("code-303", "title-145");
        assert_eq!(item.status, PlanDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_plan_draft("code-303", "title-145");
        if item.move_to(PlanDraftStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanDraftStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_plan_draft("code-303", "title-145");
        item.status = PlanDraftStatus::Archived;
        assert!(item.move_to(PlanDraftStatus::Open).is_err(), "expected move to fail");
    }
}
