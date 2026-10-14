use super::*;

use acme_kit::text;

/// Where a pending canary adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingCanaryAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingCanaryAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingCanaryAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending canary adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryAdjustment {
    pub code: String,
    pub title: String,
    pub source: String,
    pub visible: bool,
    pub created_at: i64,
    pub grace_days: i64,
    pub tenant_id: String,
    pub fingerprint: String,
    pub namespace: String,
    pub status: PendingCanaryAdjustmentStatus,
}

impl PendingCanaryAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingCanaryAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingCanaryAdjustmentStatus) -> bool {
        match self.status {
            PendingCanaryAdjustmentStatus::Created => {
                match next {
                    PendingCanaryAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryAdjustmentStatus::Validated => {
                match next {
                    PendingCanaryAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingCanaryAdjustmentStatus::Published => {
                match next {
                    PendingCanaryAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingCanaryAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending canary adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingCanaryAdjustment {
        PendingCanaryAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            visible: self.visible,
            created_at: self.created_at,
            grace_days: self.grace_days,
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending canary adjustment in status created.
pub fn new_pending_canary_adjustment(code: &str, title: &str) -> PendingCanaryAdjustment {
    PendingCanaryAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        visible: false,
        created_at: 0,
        grace_days: 0,
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        namespace: "".to_string(),
        status: PendingCanaryAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_canary_adjustment("code-649", "title-865");
        assert_eq!(item.status, PendingCanaryAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_pending_canary_adjustment("code-649", "title-865");
        if item.move_to(PendingCanaryAdjustmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingCanaryAdjustmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_canary_adjustment("code-649", "title-865");
        item.status = PendingCanaryAdjustmentStatus::Retired;
        assert!(
            item.move_to(PendingCanaryAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
