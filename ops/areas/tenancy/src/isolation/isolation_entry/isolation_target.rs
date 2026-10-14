use super::*;

use acme_kit::text;

/// Where a isolation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation target as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTarget {
    pub code: String,
    pub title: String,
    pub source: String,
    pub region: String,
    pub segment: String,
    pub amount: f64,
    pub status: IsolationTargetStatus,
}

impl IsolationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationTargetStatus) -> bool {
        match self.status {
            IsolationTargetStatus::Created => {
                return next == IsolationTargetStatus::Published;
            }
            IsolationTargetStatus::Validated => {
                return next == IsolationTargetStatus::Published;
            }
            IsolationTargetStatus::Published => {
                return next == IsolationTargetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation target", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> IsolationTarget {
        IsolationTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            segment: self.segment.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation target in status created.
pub fn new_isolation_target(code: &str, title: &str) -> IsolationTarget {
    IsolationTarget {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        region: "".to_string(),
        segment: "".to_string(),
        amount: 0.0,
        status: IsolationTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_target("code-636", "title-341");
        assert_eq!(item.status, IsolationTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_isolation_target("code-636", "title-341");
        if item.move_to(IsolationTargetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationTargetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_target("code-636", "title-341");
        item.status = IsolationTargetStatus::Retired;
        assert!(
            item.move_to(IsolationTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
