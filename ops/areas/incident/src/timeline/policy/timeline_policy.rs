use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a timeline policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelinePolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TimelinePolicyStatus {
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

impl std::fmt::Display for TimelinePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePolicy {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub audience: String,
    pub floor_value: f64,
    pub expires_at: i64,
    pub cluster: String,
    pub minimum: f64,
    pub locale: String,
    pub status: TimelinePolicyStatus,
}

impl TimelinePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelinePolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelinePolicyStatus) -> bool {
        match self.status {
            TimelinePolicyStatus::New => {
                return next == TimelinePolicyStatus::Queued || next == TimelinePolicyStatus::Running;
            }
            TimelinePolicyStatus::Queued => {
                match next {
                    TimelinePolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePolicyStatus::Running => {
                match next {
                    TimelinePolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePolicyStatus::Done => {
                return next == TimelinePolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelinePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "timeline policy", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.minimum + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelinePolicy {
        TimelinePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            audience: self.audience.clone(),
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a timeline policy in status new.
pub fn new_timeline_policy(code: &str, label: &str) -> TimelinePolicy {
    TimelinePolicy {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        audience: "".to_string(),
        floor_value: 0.0,
        expires_at: 0,
        cluster: "".to_string(),
        minimum: 0.0,
        locale: "".to_string(),
        status: TimelinePolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_timeline_policy("code-338", "label-410");
        assert_eq!(item.status, TimelinePolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_timeline_policy("code-338", "label-410");
        if item.move_to(TimelinePolicyStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelinePolicyStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_timeline_policy("code-338", "label-410");
        item.status = TimelinePolicyStatus::Failed;
        assert!(
            item.move_to(TimelinePolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
