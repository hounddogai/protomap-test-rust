use super::*;

use acme_kit::text;

/// Where a evidence rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EvidenceRollupStatus {
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

impl std::fmt::Display for EvidenceRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence rollup as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRollup {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub threshold: f64,
    pub cost: f64,
    pub ratio: f64,
    pub budget: f64,
    pub category: String,
    pub status: EvidenceRollupStatus,
}

impl EvidenceRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceRollupStatus) -> bool {
        match self.status {
            EvidenceRollupStatus::Open => {
                match next {
                    EvidenceRollupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRollupStatus::InReview => {
                match next {
                    EvidenceRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRollupStatus::Resolved => {
                return next == EvidenceRollupStatus::Reopened || next == EvidenceRollupStatus::Archived;
            }
            EvidenceRollupStatus::Reopened => {
                match next {
                    EvidenceRollupStatus::Archived => {
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

    pub fn move_to(&mut self, next: EvidenceRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.cost + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceRollup {
        EvidenceRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            cost: self.cost,
            ratio: self.ratio,
            budget: self.budget,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a evidence rollup in status open.
pub fn new_evidence_rollup(code: &str, name: &str) -> EvidenceRollup {
    EvidenceRollup {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        threshold: 0.0,
        cost: 0.0,
        ratio: 0.0,
        budget: 0.0,
        category: "".to_string(),
        status: EvidenceRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_evidence_rollup("code-535", "name-78");
        assert_eq!(item.status, EvidenceRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_evidence_rollup("code-535", "name-78");
        if item.move_to(EvidenceRollupStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceRollupStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_evidence_rollup("code-535", "name-78");
        item.status = EvidenceRollupStatus::Archived;
        assert!(
            item.move_to(EvidenceRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
