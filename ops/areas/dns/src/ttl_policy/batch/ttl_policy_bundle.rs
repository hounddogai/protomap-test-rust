use super::*;

use acme_kit::text;

/// Where a ttl policy bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TtlPolicyBundleStatus {
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

impl std::fmt::Display for TtlPolicyBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy bundle as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundle {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub visible: bool,
    pub tenant_id: String,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub attempts: i64,
    pub phone: String,
    pub threshold: f64,
    pub contact_email: String,
    pub status: TtlPolicyBundleStatus,
}

impl TtlPolicyBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyBundleStatus) -> bool {
        match self.status {
            TtlPolicyBundleStatus::New => {
                return next == TtlPolicyBundleStatus::Queued;
            }
            TtlPolicyBundleStatus::Queued => {
                return next == TtlPolicyBundleStatus::Running || next == TtlPolicyBundleStatus::Failed;
            }
            TtlPolicyBundleStatus::Running => {
                match next {
                    TtlPolicyBundleStatus::Done | TtlPolicyBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyBundleStatus::Done => {
                match next {
                    TtlPolicyBundleStatus::Failed => {
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

    pub fn move_to(&mut self, next: TtlPolicyBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quota_units as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyBundle {
        TtlPolicyBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            attempts: self.attempts,
            phone: self.phone.clone(),
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a ttl policy bundle in status new.
pub fn new_ttl_policy_bundle(code: &str, label: &str) -> TtlPolicyBundle {
    TtlPolicyBundle {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        visible: false,
        tenant_id: "".to_string(),
        quota_units: 0,
        rotation_days: 0,
        attempts: 0,
        phone: "".to_string(),
        threshold: 0.0,
        contact_email: "".to_string(),
        status: TtlPolicyBundleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_ttl_policy_bundle("code-448", "label-512");
        assert_eq!(item.status, TtlPolicyBundleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_ttl_policy_bundle("code-448", "label-512");
        if item.move_to(TtlPolicyBundleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyBundleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_ttl_policy_bundle("code-448", "label-512");
        item.status = TtlPolicyBundleStatus::Failed;
        assert!(
            item.move_to(TtlPolicyBundleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
