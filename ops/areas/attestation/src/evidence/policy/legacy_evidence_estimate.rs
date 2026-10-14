use super::*;

use acme_kit::text;

/// Where a legacy evidence estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyEvidenceEstimateStatus {
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

impl std::fmt::Display for LegacyEvidenceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy evidence estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceEstimate {
    pub code: String,
    pub name: String,
    pub host: String,
    pub locale: String,
    pub minimum: f64,
    pub cluster: String,
    pub limit: i64,
    pub version: i64,
    pub budget: f64,
    pub status: LegacyEvidenceEstimateStatus,
}

impl LegacyEvidenceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceEstimateStatus) -> bool {
        match self.status {
            LegacyEvidenceEstimateStatus::New => {
                match next {
                    LegacyEvidenceEstimateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceEstimateStatus::Queued => {
                match next {
                    LegacyEvidenceEstimateStatus::Running | LegacyEvidenceEstimateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceEstimateStatus::Running => {
                match next {
                    LegacyEvidenceEstimateStatus::Done | LegacyEvidenceEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyEvidenceEstimateStatus::Done => {
                return next == LegacyEvidenceEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEvidenceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.version as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceEstimate {
        LegacyEvidenceEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            cluster: self.cluster.clone(),
            limit: self.limit,
            version: self.version,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy evidence estimate in status new.
pub fn new_legacy_evidence_estimate(code: &str, name: &str) -> LegacyEvidenceEstimate {
    LegacyEvidenceEstimate {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        cluster: "".to_string(),
        limit: 0,
        version: 0,
        budget: 0.0,
        status: LegacyEvidenceEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_evidence_estimate("code-787", "name-468");
        assert_eq!(item.status, LegacyEvidenceEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_evidence_estimate("code-787", "name-468");
        if item.move_to(LegacyEvidenceEstimateStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyEvidenceEstimateStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_evidence_estimate("code-787", "name-468");
        item.status = LegacyEvidenceEstimateStatus::Failed;
        assert!(
            item.move_to(LegacyEvidenceEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
