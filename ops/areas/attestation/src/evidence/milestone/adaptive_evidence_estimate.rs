use super::*;

use acme_kit::text;

/// Where a adaptive evidence estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveEvidenceEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveEvidenceEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveEvidenceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive evidence estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceEstimate {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub margin: f64,
    pub source: String,
    pub cost: f64,
    pub status: AdaptiveEvidenceEstimateStatus,
}

impl AdaptiveEvidenceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveEvidenceEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveEvidenceEstimateStatus) -> bool {
        match self.status {
            AdaptiveEvidenceEstimateStatus::Created => {
                return next == AdaptiveEvidenceEstimateStatus::Published;
            }
            AdaptiveEvidenceEstimateStatus::Validated => {
                return next == AdaptiveEvidenceEstimateStatus::Published
                    || next == AdaptiveEvidenceEstimateStatus::Retired;
            }
            AdaptiveEvidenceEstimateStatus::Published => {
                return next == AdaptiveEvidenceEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveEvidenceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive evidence estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.score * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveEvidenceEstimate {
        AdaptiveEvidenceEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            margin: self.margin,
            source: self.source.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive evidence estimate in status created.
pub fn new_adaptive_evidence_estimate(code: &str, label: &str) -> AdaptiveEvidenceEstimate {
    AdaptiveEvidenceEstimate {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        margin: 0.0,
        source: "".to_string(),
        cost: 0.0,
        status: AdaptiveEvidenceEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_evidence_estimate("code-273", "label-838");
        assert_eq!(item.status, AdaptiveEvidenceEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_evidence_estimate("code-273", "label-838");
        if item.move_to(AdaptiveEvidenceEstimateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveEvidenceEstimateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_evidence_estimate("code-273", "label-838");
        item.status = AdaptiveEvidenceEstimateStatus::Retired;
        assert!(
            item.move_to(AdaptiveEvidenceEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
