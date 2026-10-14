use super::*;

use acme_kit::text;

/// Where a legacy evidence claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyEvidenceClaimStatus {
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

impl std::fmt::Display for LegacyEvidenceClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy evidence claim as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaim {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub grace_days: i64,
    pub notes: String,
    pub host: String,
    pub status: LegacyEvidenceClaimStatus,
}

impl LegacyEvidenceClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceClaimStatus) -> bool {
        match self.status {
            LegacyEvidenceClaimStatus::New => {
                return next == LegacyEvidenceClaimStatus::Running || next == LegacyEvidenceClaimStatus::Failed;
            }
            LegacyEvidenceClaimStatus::Queued => {
                match next {
                    LegacyEvidenceClaimStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceClaimStatus::Running => {
                return next == LegacyEvidenceClaimStatus::Done || next == LegacyEvidenceClaimStatus::Failed;
            }
            LegacyEvidenceClaimStatus::Done => {
                return next == LegacyEvidenceClaimStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEvidenceClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceClaim {
        LegacyEvidenceClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            grace_days: self.grace_days,
            notes: self.notes.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy evidence claim in status new.
pub fn new_legacy_evidence_claim(code: &str, title: &str) -> LegacyEvidenceClaim {
    LegacyEvidenceClaim {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        grace_days: 0,
        notes: "".to_string(),
        host: "".to_string(),
        status: LegacyEvidenceClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_evidence_claim("code-982", "title-163");
        assert_eq!(item.status, LegacyEvidenceClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_evidence_claim("code-982", "title-163");
        if item.move_to(LegacyEvidenceClaimStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyEvidenceClaimStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_evidence_claim("code-982", "title-163");
        item.status = LegacyEvidenceClaimStatus::Failed;
        assert!(
            item.move_to(LegacyEvidenceClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
