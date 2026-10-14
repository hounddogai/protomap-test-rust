use super::*;

use acme_kit::text;

/// Where a strict delegation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDelegationTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictDelegationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictDelegationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict delegation target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTarget {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub contact_email: String,
    pub amount: f64,
    pub summary: String,
    pub threshold: f64,
    pub visible: bool,
    pub sequence: i64,
    pub priority: i64,
    pub minimum: f64,
    pub status: StrictDelegationTargetStatus,
}

impl StrictDelegationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDelegationTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDelegationTargetStatus) -> bool {
        match self.status {
            StrictDelegationTargetStatus::Created => {
                match next {
                    StrictDelegationTargetStatus::Validated | StrictDelegationTargetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationTargetStatus::Validated => {
                match next {
                    StrictDelegationTargetStatus::Published | StrictDelegationTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationTargetStatus::Published => {
                match next {
                    StrictDelegationTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictDelegationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict delegation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict delegation target", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.version as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDelegationTarget {
        StrictDelegationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            summary: self.summary.clone(),
            threshold: self.threshold,
            visible: self.visible,
            sequence: self.sequence,
            priority: self.priority,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict delegation target in status created.
pub fn new_strict_delegation_target(code: &str, name: &str) -> StrictDelegationTarget {
    StrictDelegationTarget {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        contact_email: "".to_string(),
        amount: 0.0,
        summary: "".to_string(),
        threshold: 0.0,
        visible: false,
        sequence: 0,
        priority: 0,
        minimum: 0.0,
        status: StrictDelegationTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_delegation_target("code-865", "name-604");
        assert_eq!(item.status, StrictDelegationTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_delegation_target("code-865", "name-604");
        if item.move_to(StrictDelegationTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictDelegationTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_delegation_target("code-865", "name-604");
        item.status = StrictDelegationTargetStatus::Retired;
        assert!(
            item.move_to(StrictDelegationTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
