use super::*;

use acme_kit::text;

/// Where a primary policy rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPolicyRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryPolicyRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PrimaryPolicyRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary policy rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPolicyRollup {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub source: String,
    pub version: i64,
    pub status: PrimaryPolicyRollupStatus,
}

impl PrimaryPolicyRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPolicyRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPolicyRollupStatus) -> bool {
        match self.status {
            PrimaryPolicyRollupStatus::Draft => {
                return next == PrimaryPolicyRollupStatus::Active;
            }
            PrimaryPolicyRollupStatus::Pending => {
                return next == PrimaryPolicyRollupStatus::Active || next == PrimaryPolicyRollupStatus::Closed;
            }
            PrimaryPolicyRollupStatus::Active => {
                return next == PrimaryPolicyRollupStatus::Closed;
            }
            PrimaryPolicyRollupStatus::Suspended => {
                match next {
                    PrimaryPolicyRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryPolicyRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary policy rollup {} cannot move from {} to {}",
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

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> PrimaryPolicyRollup {
        PrimaryPolicyRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            source: self.source.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary policy rollup in status draft.
pub fn new_primary_policy_rollup(code: &str, name: &str) -> PrimaryPolicyRollup {
    PrimaryPolicyRollup {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        source: "".to_string(),
        version: 0,
        status: PrimaryPolicyRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_policy_rollup("code-58", "name-73");
        assert_eq!(item.status, PrimaryPolicyRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_primary_policy_rollup("code-58", "name-73");
        if item.move_to(PrimaryPolicyRollupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPolicyRollupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_policy_rollup("code-58", "name-73");
        item.status = PrimaryPolicyRollupStatus::Closed;
        assert!(
            item.move_to(PrimaryPolicyRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
