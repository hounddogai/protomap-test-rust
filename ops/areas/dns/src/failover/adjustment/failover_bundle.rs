use super::*;

use acme_kit::text;

/// Where a failover bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverBundleStatus {
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

impl std::fmt::Display for FailoverBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBundle {
    pub code: String,
    pub name: String,
    pub country: String,
    pub pinned: bool,
    pub currency: String,
    pub threshold: f64,
    pub status: FailoverBundleStatus,
}

impl FailoverBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverBundleStatus) -> bool {
        match self.status {
            FailoverBundleStatus::New => {
                return next == FailoverBundleStatus::Running || next == FailoverBundleStatus::Failed;
            }
            FailoverBundleStatus::Queued => {
                return next == FailoverBundleStatus::Running || next == FailoverBundleStatus::Failed;
            }
            FailoverBundleStatus::Running => {
                return next == FailoverBundleStatus::Done;
            }
            FailoverBundleStatus::Done => {
                return next == FailoverBundleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover bundle {} cannot move from {} to {}",
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
        self.threshold
    }

    pub fn copy(&self) -> FailoverBundle {
        FailoverBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            pinned: self.pinned,
            currency: self.currency.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a failover bundle in status new.
pub fn new_failover_bundle(code: &str, name: &str) -> FailoverBundle {
    FailoverBundle {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        pinned: false,
        currency: "".to_string(),
        threshold: 0.0,
        status: FailoverBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_bundle("code-475", "name-346");
        assert_eq!(item.status, FailoverBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_failover_bundle("code-475", "name-346");
        if item.move_to(FailoverBundleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverBundleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_bundle("code-475", "name-346");
        item.status = FailoverBundleStatus::Failed;
        assert!(
            item.move_to(FailoverBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
