use super::*;

use acme_kit::text;

/// Where a scheduled escalation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEscalationPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledEscalationPolicyStatus {
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

impl std::fmt::Display for ScheduledEscalationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled escalation policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEscalationPolicy {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub sequence: i64,
    pub source: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub tenant_id: String,
    pub locale: String,
    pub minimum: f64,
    pub status: ScheduledEscalationPolicyStatus,
}

impl ScheduledEscalationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEscalationPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEscalationPolicyStatus) -> bool {
        match self.status {
            ScheduledEscalationPolicyStatus::Idle => {
                match next {
                    ScheduledEscalationPolicyStatus::Ready | ScheduledEscalationPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEscalationPolicyStatus::Warming => {
                return next == ScheduledEscalationPolicyStatus::Draining;
            }
            ScheduledEscalationPolicyStatus::Ready => {
                match next {
                    ScheduledEscalationPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEscalationPolicyStatus::Draining => {
                match next {
                    ScheduledEscalationPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledEscalationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled escalation policy {} cannot move from {} to {}",
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
        format!("{} {}", "scheduled escalation policy", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.minimum * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledEscalationPolicy {
        ScheduledEscalationPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            sequence: self.sequence,
            source: self.source.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled escalation policy in status idle.
pub fn new_scheduled_escalation_policy(code: &str, name: &str) -> ScheduledEscalationPolicy {
    ScheduledEscalationPolicy {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        sequence: 0,
        source: "".to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        status: ScheduledEscalationPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_escalation_policy("code-300", "name-349");
        assert_eq!(item.status, ScheduledEscalationPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_escalation_policy("code-300", "name-349");
        if item.move_to(ScheduledEscalationPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledEscalationPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_escalation_policy("code-300", "name-349");
        item.status = ScheduledEscalationPolicyStatus::Stopped;
        assert!(
            item.move_to(ScheduledEscalationPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
