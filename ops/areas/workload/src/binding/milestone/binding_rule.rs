use super::*;

use acme_kit::text;

/// Where a binding rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingRuleStatus {
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

impl std::fmt::Display for BindingRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRule {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub ends_at: i64,
    pub starts_at: i64,
    pub segment: String,
    pub checksum: String,
    pub scope: String,
    pub created_at: i64,
    pub floor_value: f64,
    pub expires_at: i64,
    pub status: BindingRuleStatus,
}

impl BindingRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingRuleStatus) -> bool {
        match self.status {
            BindingRuleStatus::Idle => {
                return next == BindingRuleStatus::Warming;
            }
            BindingRuleStatus::Warming => {
                match next {
                    BindingRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingRuleStatus::Ready => {
                return next == BindingRuleStatus::Draining || next == BindingRuleStatus::Stopped;
            }
            BindingRuleStatus::Draining => {
                return next == BindingRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingRule {
        BindingRule {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            scope: self.scope.clone(),
            created_at: self.created_at,
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a binding rule in status idle.
pub fn new_binding_rule(code: &str, name: &str) -> BindingRule {
    BindingRule {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        ends_at: 0,
        starts_at: 0,
        segment: "".to_string(),
        checksum: "".to_string(),
        scope: "".to_string(),
        created_at: 0,
        floor_value: 0.0,
        expires_at: 0,
        status: BindingRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_rule("code-649", "name-711");
        assert_eq!(item.status, BindingRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_binding_rule("code-649", "name-711");
        if item.move_to(BindingRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_rule("code-649", "name-711");
        item.status = BindingRuleStatus::Stopped;
        assert!(item.move_to(BindingRuleStatus::Idle).is_err(), "expected move to fail");
    }
}
