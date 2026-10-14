use super::*;

use acme_kit::text;

/// Where a adaptive retention override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRetentionOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveRetentionOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for AdaptiveRetentionOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive retention override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionOverride {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub weight: f64,
    pub label: String,
    pub quantity: i64,
    pub scope: String,
    pub cost: f64,
    pub tenant_id: String,
    pub ceiling: f64,
    pub status: AdaptiveRetentionOverrideStatus,
}

impl AdaptiveRetentionOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRetentionOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRetentionOverrideStatus) -> bool {
        match self.status {
            AdaptiveRetentionOverrideStatus::Planned => {
                return next == AdaptiveRetentionOverrideStatus::Completed;
            }
            AdaptiveRetentionOverrideStatus::Scheduled => {
                return next == AdaptiveRetentionOverrideStatus::InProgress
                    || next == AdaptiveRetentionOverrideStatus::Completed;
            }
            AdaptiveRetentionOverrideStatus::InProgress => {
                match next {
                    AdaptiveRetentionOverrideStatus::Completed | AdaptiveRetentionOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRetentionOverrideStatus::Completed => {
                return next == AdaptiveRetentionOverrideStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRetentionOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive retention override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.weight + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRetentionOverride {
        AdaptiveRetentionOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            weight: self.weight,
            label: self.label.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive retention override in status planned.
pub fn new_adaptive_retention_override(code: &str, title: &str) -> AdaptiveRetentionOverride {
    AdaptiveRetentionOverride {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        weight: 0.0,
        label: "".to_string(),
        quantity: 0,
        scope: "".to_string(),
        cost: 0.0,
        tenant_id: "".to_string(),
        ceiling: 0.0,
        status: AdaptiveRetentionOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_retention_override("code-503", "title-40");
        assert_eq!(item.status, AdaptiveRetentionOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_adaptive_retention_override("code-503", "title-40");
        if item.move_to(AdaptiveRetentionOverrideStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRetentionOverrideStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_retention_override("code-503", "title-40");
        item.status = AdaptiveRetentionOverrideStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveRetentionOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
