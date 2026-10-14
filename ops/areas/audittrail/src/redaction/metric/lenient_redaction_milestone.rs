use super::*;

use acme_kit::text;

/// Where a lenient redaction milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRedactionMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientRedactionMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientRedactionMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient redaction milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestone {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub version: i64,
    pub limit: i64,
    pub phone: String,
    pub status: LenientRedactionMilestoneStatus,
}

impl LenientRedactionMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRedactionMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRedactionMilestoneStatus) -> bool {
        match self.status {
            LenientRedactionMilestoneStatus::Created => {
                match next {
                    LenientRedactionMilestoneStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionMilestoneStatus::Validated => {
                match next {
                    LenientRedactionMilestoneStatus::Published | LenientRedactionMilestoneStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionMilestoneStatus::Published => {
                match next {
                    LenientRedactionMilestoneStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientRedactionMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient redaction milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.limit as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRedactionMilestone {
        LenientRedactionMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            version: self.version,
            limit: self.limit,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient redaction milestone in status created.
pub fn new_lenient_redaction_milestone(code: &str, name: &str) -> LenientRedactionMilestone {
    LenientRedactionMilestone {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        version: 0,
        limit: 0,
        phone: "".to_string(),
        status: LenientRedactionMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_redaction_milestone("code-79", "name-614");
        assert_eq!(item.status, LenientRedactionMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_lenient_redaction_milestone("code-79", "name-614");
        if item.move_to(LenientRedactionMilestoneStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRedactionMilestoneStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_redaction_milestone("code-79", "name-614");
        item.status = LenientRedactionMilestoneStatus::Retired;
        assert!(
            item.move_to(LenientRedactionMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
