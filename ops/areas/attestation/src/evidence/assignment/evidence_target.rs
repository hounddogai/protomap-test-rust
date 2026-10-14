use super::*;

use acme_kit::text;

/// Where a evidence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EvidenceTargetStatus {
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

impl std::fmt::Display for EvidenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence target as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTarget {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub retries: i64,
    pub locale: String,
    pub status: EvidenceTargetStatus,
}

impl EvidenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceTargetStatus) -> bool {
        match self.status {
            EvidenceTargetStatus::New => {
                return next == EvidenceTargetStatus::Running || next == EvidenceTargetStatus::Failed;
            }
            EvidenceTargetStatus::Queued => {
                return next == EvidenceTargetStatus::Done;
            }
            EvidenceTargetStatus::Running => {
                match next {
                    EvidenceTargetStatus::Done | EvidenceTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceTargetStatus::Done => {
                match next {
                    EvidenceTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: EvidenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence target", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceTarget {
        EvidenceTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            retries: self.retries,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a evidence target in status new.
pub fn new_evidence_target(code: &str, title: &str) -> EvidenceTarget {
    EvidenceTarget {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        retries: 0,
        locale: "".to_string(),
        status: EvidenceTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_evidence_target("code-949", "title-970");
        assert_eq!(item.status, EvidenceTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_evidence_target("code-949", "title-970");
        if item.move_to(EvidenceTargetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceTargetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_evidence_target("code-949", "title-970");
        item.status = EvidenceTargetStatus::Failed;
        assert!(
            item.move_to(EvidenceTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
