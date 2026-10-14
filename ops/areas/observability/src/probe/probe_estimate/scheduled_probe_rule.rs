use super::*;

use acme_kit::text;

/// Where a scheduled probe rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledProbeRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledProbeRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledProbeRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled probe rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeRule {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub pinned: bool,
    pub namespace: String,
    pub budget: f64,
    pub starts_at: i64,
    pub label: String,
    pub discount: f64,
    pub segment: String,
    pub port: i64,
    pub status: ScheduledProbeRuleStatus,
}

impl ScheduledProbeRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledProbeRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledProbeRuleStatus) -> bool {
        match self.status {
            ScheduledProbeRuleStatus::Proposed => {
                return next == ScheduledProbeRuleStatus::Rejected;
            }
            ScheduledProbeRuleStatus::Approved => {
                match next {
                    ScheduledProbeRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeRuleStatus::Rejected => {
                match next {
                    ScheduledProbeRuleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledProbeRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled probe rule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled probe rule", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.budget + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledProbeRule {
        ScheduledProbeRule {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            pinned: self.pinned,
            namespace: self.namespace.clone(),
            budget: self.budget,
            starts_at: self.starts_at,
            label: self.label.clone(),
            discount: self.discount,
            segment: self.segment.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled probe rule in status proposed.
pub fn new_scheduled_probe_rule(code: &str, name: &str) -> ScheduledProbeRule {
    ScheduledProbeRule {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        pinned: false,
        namespace: "".to_string(),
        budget: 0.0,
        starts_at: 0,
        label: "".to_string(),
        discount: 0.0,
        segment: "".to_string(),
        port: 0,
        status: ScheduledProbeRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_probe_rule("code-363", "name-543");
        assert_eq!(item.status, ScheduledProbeRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_scheduled_probe_rule("code-363", "name-543");
        if item.move_to(ScheduledProbeRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledProbeRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_probe_rule("code-363", "name-543");
        item.status = ScheduledProbeRuleStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledProbeRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
