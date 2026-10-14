use super::*;

use acme_kit::text;

/// Where a canary estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CanaryEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CanaryEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary estimate as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimate {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub weight: f64,
    pub cluster: String,
    pub status: CanaryEstimateStatus,
}

impl CanaryEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryEstimateStatus) -> bool {
        match self.status {
            CanaryEstimateStatus::Created => {
                match next {
                    CanaryEstimateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryEstimateStatus::Validated => {
                match next {
                    CanaryEstimateStatus::Published | CanaryEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryEstimateStatus::Published => {
                match next {
                    CanaryEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: CanaryEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.weight * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryEstimate {
        CanaryEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            weight: self.weight,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a canary estimate in status created.
pub fn new_canary_estimate(code: &str, title: &str) -> CanaryEstimate {
    CanaryEstimate {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        weight: 0.0,
        cluster: "".to_string(),
        status: CanaryEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_canary_estimate("code-522", "title-139");
        assert_eq!(item.status, CanaryEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_canary_estimate("code-522", "title-139");
        if item.move_to(CanaryEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_canary_estimate("code-522", "title-139");
        item.status = CanaryEstimateStatus::Retired;
        assert!(
            item.move_to(CanaryEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
