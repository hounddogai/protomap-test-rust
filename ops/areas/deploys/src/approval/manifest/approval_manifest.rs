use super::*;

use acme_kit::text;

/// Where a approval manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalManifest {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub starts_at: i64,
    pub batch_size: i64,
    pub status: ApprovalManifestStatus,
}

impl ApprovalManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalManifestStatus) -> bool {
        match self.status {
            ApprovalManifestStatus::Idle => {
                match next {
                    ApprovalManifestStatus::Warming | ApprovalManifestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalManifestStatus::Warming => {
                match next {
                    ApprovalManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalManifestStatus::Ready => {
                match next {
                    ApprovalManifestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalManifestStatus::Draining => {
                return next == ApprovalManifestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalManifest {
        ApprovalManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            starts_at: self.starts_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a approval manifest in status idle.
pub fn new_approval_manifest(code: &str, name: &str) -> ApprovalManifest {
    ApprovalManifest {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        attempts: 0,
        starts_at: 0,
        batch_size: 0,
        status: ApprovalManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_manifest("code-869", "name-732");
        assert_eq!(item.status, ApprovalManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_approval_manifest("code-869", "name-732");
        if item.move_to(ApprovalManifestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalManifestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_manifest("code-869", "name-732");
        item.status = ApprovalManifestStatus::Stopped;
        assert!(
            item.move_to(ApprovalManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
