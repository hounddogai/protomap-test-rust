use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a action item segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemSegmentStatus {
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

impl std::fmt::Display for ActionItemSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegment {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub summary: String,
    pub issuer: String,
    pub label: String,
    pub principal: String,
    pub score: f64,
    pub status: ActionItemSegmentStatus,
}

impl ActionItemSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemSegmentStatus) -> bool {
        match self.status {
            ActionItemSegmentStatus::Idle => {
                match next {
                    ActionItemSegmentStatus::Warming | ActionItemSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSegmentStatus::Warming => {
                return next == ActionItemSegmentStatus::Stopped;
            }
            ActionItemSegmentStatus::Ready => {
                match next {
                    ActionItemSegmentStatus::Draining | ActionItemSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSegmentStatus::Draining => {
                match next {
                    ActionItemSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ActionItemSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "action item segment", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ActionItemSegment {
        ActionItemSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            summary: self.summary.clone(),
            issuer: self.issuer.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a action item segment in status idle.
pub fn new_action_item_segment(code: &str, name: &str) -> ActionItemSegment {
    ActionItemSegment {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        summary: "".to_string(),
        issuer: "".to_string(),
        label: "".to_string(),
        principal: "".to_string(),
        score: 0.0,
        status: ActionItemSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_action_item_segment("code-444", "name-419");
        assert_eq!(item.status, ActionItemSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_action_item_segment("code-444", "name-419");
        if item.move_to(ActionItemSegmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemSegmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_action_item_segment("code-444", "name-419");
        item.status = ActionItemSegmentStatus::Stopped;
        assert!(
            item.move_to(ActionItemSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
