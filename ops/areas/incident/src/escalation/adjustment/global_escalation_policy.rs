use super::*;

use acme_kit::text;

/// Where a global escalation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEscalationPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalEscalationPolicyStatus {
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

impl std::fmt::Display for GlobalEscalationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global escalation policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationPolicy {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub updated_at: i64,
    pub timezone: String,
    pub summary: String,
    pub status: GlobalEscalationPolicyStatus,
}

impl GlobalEscalationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEscalationPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEscalationPolicyStatus) -> bool {
        match self.status {
            GlobalEscalationPolicyStatus::New => {
                return next == GlobalEscalationPolicyStatus::Queued;
            }
            GlobalEscalationPolicyStatus::Queued => {
                match next {
                    GlobalEscalationPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationPolicyStatus::Running => {
                match next {
                    GlobalEscalationPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationPolicyStatus::Done => {
                return next == GlobalEscalationPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalEscalationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global escalation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEscalationPolicy {
        GlobalEscalationPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            updated_at: self.updated_at,
            timezone: self.timezone.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global escalation policy in status new.
pub fn new_global_escalation_policy(code: &str, label: &str) -> GlobalEscalationPolicy {
    GlobalEscalationPolicy {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        updated_at: 0,
        timezone: "".to_string(),
        summary: "".to_string(),
        status: GlobalEscalationPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_escalation_policy("code-211", "label-12");
        assert_eq!(item.status, GlobalEscalationPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_global_escalation_policy("code-211", "label-12");
        if item.move_to(GlobalEscalationPolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEscalationPolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_escalation_policy("code-211", "label-12");
        item.status = GlobalEscalationPolicyStatus::Failed;
        assert!(
            item.move_to(GlobalEscalationPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
