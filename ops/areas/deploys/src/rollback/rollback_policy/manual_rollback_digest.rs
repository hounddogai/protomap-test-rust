use super::*;

use acme_kit::text;

/// Where a manual rollback digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRollbackDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualRollbackDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualRollbackDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rollback digest as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackDigest {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub minimum: f64,
    pub name: String,
    pub visible: bool,
    pub reviewer: String,
    pub cluster: String,
    pub cost: f64,
    pub issuer: String,
    pub status: ManualRollbackDigestStatus,
}

impl ManualRollbackDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRollbackDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRollbackDigestStatus) -> bool {
        match self.status {
            ManualRollbackDigestStatus::New => {
                return next == ManualRollbackDigestStatus::Running || next == ManualRollbackDigestStatus::Failed;
            }
            ManualRollbackDigestStatus::Queued => {
                return next == ManualRollbackDigestStatus::Done;
            }
            ManualRollbackDigestStatus::Running => {
                return next == ManualRollbackDigestStatus::Failed;
            }
            ManualRollbackDigestStatus::Done => {
                return next == ManualRollbackDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRollbackDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollback digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        (self.cost * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRollbackDigest {
        ManualRollbackDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            minimum: self.minimum,
            name: self.name.clone(),
            visible: self.visible,
            reviewer: self.reviewer.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual rollback digest in status new.
pub fn new_manual_rollback_digest(code: &str, label: &str) -> ManualRollbackDigest {
    ManualRollbackDigest {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        minimum: 0.0,
        name: "".to_string(),
        visible: false,
        reviewer: "".to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        issuer: "".to_string(),
        status: ManualRollbackDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_rollback_digest("code-8", "label-178");
        assert_eq!(item.status, ManualRollbackDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_rollback_digest("code-8", "label-178");
        if item.move_to(ManualRollbackDigestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRollbackDigestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_rollback_digest("code-8", "label-178");
        item.status = ManualRollbackDigestStatus::Failed;
        assert!(
            item.move_to(ManualRollbackDigestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
