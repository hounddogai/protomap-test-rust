use super::*;

use acme_kit::text;

/// Where a primary evidence plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidencePlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryEvidencePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryEvidencePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary evidence plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidencePlan {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub channel: String,
    pub ratio: f64,
    pub version: i64,
    pub amount: f64,
    pub batch_size: i64,
    pub status: PrimaryEvidencePlanStatus,
}

impl PrimaryEvidencePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidencePlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidencePlanStatus) -> bool {
        match self.status {
            PrimaryEvidencePlanStatus::Created => {
                match next {
                    PrimaryEvidencePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidencePlanStatus::Validated => {
                match next {
                    PrimaryEvidencePlanStatus::Published | PrimaryEvidencePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidencePlanStatus::Published => {
                match next {
                    PrimaryEvidencePlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryEvidencePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.amount * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidencePlan {
        PrimaryEvidencePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            channel: self.channel.clone(),
            ratio: self.ratio,
            version: self.version,
            amount: self.amount,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary evidence plan in status created.
pub fn new_primary_evidence_plan(code: &str, name: &str) -> PrimaryEvidencePlan {
    PrimaryEvidencePlan {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        channel: "".to_string(),
        ratio: 0.0,
        version: 0,
        amount: 0.0,
        batch_size: 0,
        status: PrimaryEvidencePlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_evidence_plan("code-366", "name-183");
        assert_eq!(item.status, PrimaryEvidencePlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_primary_evidence_plan("code-366", "name-183");
        if item.move_to(PrimaryEvidencePlanStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidencePlanStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_evidence_plan("code-366", "name-183");
        item.status = PrimaryEvidencePlanStatus::Retired;
        assert!(
            item.move_to(PrimaryEvidencePlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
