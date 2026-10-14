use super::*;

use acme_kit::text;

/// Where a strict evidence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEvidenceBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictEvidenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictEvidenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict evidence budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudget {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub cluster: String,
    pub ends_at: i64,
    pub status: StrictEvidenceBudgetStatus,
}

impl StrictEvidenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEvidenceBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEvidenceBudgetStatus) -> bool {
        match self.status {
            StrictEvidenceBudgetStatus::Created => {
                return next == StrictEvidenceBudgetStatus::Validated;
            }
            StrictEvidenceBudgetStatus::Validated => {
                match next {
                    StrictEvidenceBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEvidenceBudgetStatus::Published => {
                match next {
                    StrictEvidenceBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictEvidenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict evidence budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rate - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEvidenceBudget {
        StrictEvidenceBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            cluster: self.cluster.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a strict evidence budget in status created.
pub fn new_strict_evidence_budget(code: &str, title: &str) -> StrictEvidenceBudget {
    StrictEvidenceBudget {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        cluster: "".to_string(),
        ends_at: 0,
        status: StrictEvidenceBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_evidence_budget("code-916", "title-295");
        assert_eq!(item.status, StrictEvidenceBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_evidence_budget("code-916", "title-295");
        if item.move_to(StrictEvidenceBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEvidenceBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_evidence_budget("code-916", "title-295");
        item.status = StrictEvidenceBudgetStatus::Retired;
        assert!(
            item.move_to(StrictEvidenceBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
