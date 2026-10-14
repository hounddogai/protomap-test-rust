use super::*;

use acme_kit::text;

/// Where a default ttl policy bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTtlPolicyBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultTtlPolicyBundleStatus {
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

impl std::fmt::Display for DefaultTtlPolicyBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default ttl policy bundle as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundle {
    pub code: String,
    pub label: String,
    pub description: String,
    pub segment: String,
    pub unit: String,
    pub score: f64,
    pub rotation_days: i64,
    pub cost: f64,
    pub port: i64,
    pub tags: Vec<String>,
    pub status: DefaultTtlPolicyBundleStatus,
}

impl DefaultTtlPolicyBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTtlPolicyBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTtlPolicyBundleStatus) -> bool {
        match self.status {
            DefaultTtlPolicyBundleStatus::Idle => {
                match next {
                    DefaultTtlPolicyBundleStatus::Draining | DefaultTtlPolicyBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTtlPolicyBundleStatus::Warming => {
                match next {
                    DefaultTtlPolicyBundleStatus::Ready | DefaultTtlPolicyBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTtlPolicyBundleStatus::Ready => {
                return next == DefaultTtlPolicyBundleStatus::Stopped;
            }
            DefaultTtlPolicyBundleStatus::Draining => {
                match next {
                    DefaultTtlPolicyBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultTtlPolicyBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default ttl policy bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.score + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTtlPolicyBundle {
        DefaultTtlPolicyBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            unit: self.unit.clone(),
            score: self.score,
            rotation_days: self.rotation_days,
            cost: self.cost,
            port: self.port,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default ttl policy bundle in status idle.
pub fn new_default_ttl_policy_bundle(code: &str, label: &str) -> DefaultTtlPolicyBundle {
    DefaultTtlPolicyBundle {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        unit: "".to_string(),
        score: 0.0,
        rotation_days: 0,
        cost: 0.0,
        port: 0,
        tags: Vec::new(),
        status: DefaultTtlPolicyBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_ttl_policy_bundle("code-789", "label-845");
        assert_eq!(item.status, DefaultTtlPolicyBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_default_ttl_policy_bundle("code-789", "label-845");
        if item.move_to(DefaultTtlPolicyBundleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTtlPolicyBundleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_ttl_policy_bundle("code-789", "label-845");
        item.status = DefaultTtlPolicyBundleStatus::Stopped;
        assert!(
            item.move_to(DefaultTtlPolicyBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
