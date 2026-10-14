use super::*;

use acme_kit::text;

/// Where a manual flow log digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFlowLogDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualFlowLogDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualFlowLogDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual flow log digest as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigest {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub active: bool,
    pub issuer: String,
    pub score: f64,
    pub status: ManualFlowLogDigestStatus,
}

impl ManualFlowLogDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFlowLogDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFlowLogDigestStatus) -> bool {
        match self.status {
            ManualFlowLogDigestStatus::Created => {
                return next == ManualFlowLogDigestStatus::Published || next == ManualFlowLogDigestStatus::Retired;
            }
            ManualFlowLogDigestStatus::Validated => {
                return next == ManualFlowLogDigestStatus::Retired;
            }
            ManualFlowLogDigestStatus::Published => {
                match next {
                    ManualFlowLogDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualFlowLogDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual flow log digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ManualFlowLogDigest {
        ManualFlowLogDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            active: self.active,
            issuer: self.issuer.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual flow log digest in status created.
pub fn new_manual_flow_log_digest(code: &str, name: &str) -> ManualFlowLogDigest {
    ManualFlowLogDigest {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        active: false,
        issuer: "".to_string(),
        score: 0.0,
        status: ManualFlowLogDigestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_flow_log_digest("code-473", "name-249");
        assert_eq!(item.status, ManualFlowLogDigestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_flow_log_digest("code-473", "name-249");
        if item.move_to(ManualFlowLogDigestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFlowLogDigestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_flow_log_digest("code-473", "name-249");
        item.status = ManualFlowLogDigestStatus::Retired;
        assert!(
            item.move_to(ManualFlowLogDigestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
