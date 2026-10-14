use super::*;

use acme_kit::text;

/// Where a rule adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleAdjustmentStatus {
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

impl std::fmt::Display for RuleAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAdjustment {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub margin: f64,
    pub batch_size: i64,
    pub issuer: String,
    pub floor_value: f64,
    pub cluster: String,
    pub minimum: f64,
    pub version: i64,
    pub fingerprint: String,
    pub status: RuleAdjustmentStatus,
}

impl RuleAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleAdjustmentStatus) -> bool {
        match self.status {
            RuleAdjustmentStatus::Open => {
                match next {
                    RuleAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAdjustmentStatus::InReview => {
                match next {
                    RuleAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAdjustmentStatus::Resolved => {
                match next {
                    RuleAdjustmentStatus::Reopened | RuleAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAdjustmentStatus::Reopened => {
                return next == RuleAdjustmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.weight + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleAdjustment {
        RuleAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            margin: self.margin,
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            version: self.version,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rule adjustment in status open.
pub fn new_rule_adjustment(code: &str, name: &str) -> RuleAdjustment {
    RuleAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        margin: 0.0,
        batch_size: 0,
        issuer: "".to_string(),
        floor_value: 0.0,
        cluster: "".to_string(),
        minimum: 0.0,
        version: 0,
        fingerprint: "".to_string(),
        status: RuleAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rule_adjustment("code-170", "name-190");
        assert_eq!(item.status, RuleAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_rule_adjustment("code-170", "name-190");
        if item.move_to(RuleAdjustmentStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleAdjustmentStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rule_adjustment("code-170", "name-190");
        item.status = RuleAdjustmentStatus::Archived;
        assert!(
            item.move_to(RuleAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
