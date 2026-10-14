use super::*;

use acme_kit::text;

/// Where a federation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationBundleStatus {
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

impl std::fmt::Display for FederationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundle {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub phone: String,
    pub key_id: String,
    pub tenant_id: String,
    pub source: String,
    pub status: FederationBundleStatus,
}

impl FederationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationBundleStatus) -> bool {
        match self.status {
            FederationBundleStatus::New => {
                return next == FederationBundleStatus::Queued || next == FederationBundleStatus::Done;
            }
            FederationBundleStatus::Queued => {
                match next {
                    FederationBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationBundleStatus::Running => {
                match next {
                    FederationBundleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationBundleStatus::Done => {
                return next == FederationBundleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> FederationBundle {
        FederationBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            tenant_id: self.tenant_id.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a federation bundle in status new.
pub fn new_federation_bundle(code: &str, name: &str) -> FederationBundle {
    FederationBundle {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        phone: "".to_string(),
        key_id: "".to_string(),
        tenant_id: "".to_string(),
        source: "".to_string(),
        status: FederationBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_bundle("code-290", "name-176");
        assert_eq!(item.status, FederationBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_federation_bundle("code-290", "name-176");
        if item.move_to(FederationBundleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationBundleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_bundle("code-290", "name-176");
        item.status = FederationBundleStatus::Failed;
        assert!(
            item.move_to(FederationBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
