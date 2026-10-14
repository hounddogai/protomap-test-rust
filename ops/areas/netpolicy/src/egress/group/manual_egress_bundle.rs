use super::*;

use acme_kit::text;

/// Where a manual egress bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEgressBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualEgressBundleStatus {
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

impl std::fmt::Display for ManualEgressBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual egress bundle as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEgressBundle {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub batch_size: i64,
    pub starts_at: i64,
    pub status: ManualEgressBundleStatus,
}

impl ManualEgressBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEgressBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEgressBundleStatus) -> bool {
        match self.status {
            ManualEgressBundleStatus::New => {
                match next {
                    ManualEgressBundleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEgressBundleStatus::Queued => {
                match next {
                    ManualEgressBundleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEgressBundleStatus::Running => {
                match next {
                    ManualEgressBundleStatus::Done | ManualEgressBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEgressBundleStatus::Done => {
                match next {
                    ManualEgressBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualEgressBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual egress bundle {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.batch_size as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEgressBundle {
        ManualEgressBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual egress bundle in status new.
pub fn new_manual_egress_bundle(code: &str, title: &str) -> ManualEgressBundle {
    ManualEgressBundle {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        batch_size: 0,
        starts_at: 0,
        status: ManualEgressBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_egress_bundle("code-491", "title-987");
        assert_eq!(item.status, ManualEgressBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_manual_egress_bundle("code-491", "title-987");
        if item.move_to(ManualEgressBundleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualEgressBundleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_egress_bundle("code-491", "title-987");
        item.status = ManualEgressBundleStatus::Failed;
        assert!(
            item.move_to(ManualEgressBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
