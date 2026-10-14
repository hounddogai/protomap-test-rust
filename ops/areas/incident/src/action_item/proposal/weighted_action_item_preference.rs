use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted action item preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedActionItemPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedActionItemPreferenceStatus {
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

impl std::fmt::Display for WeightedActionItemPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted action item preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemPreference {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub ceiling: f64,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub segment: String,
    pub priority: i64,
    pub title: String,
    pub principal: String,
    pub status: WeightedActionItemPreferenceStatus,
}

impl WeightedActionItemPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedActionItemPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedActionItemPreferenceStatus) -> bool {
        match self.status {
            WeightedActionItemPreferenceStatus::Idle => {
                return next == WeightedActionItemPreferenceStatus::Warming;
            }
            WeightedActionItemPreferenceStatus::Warming => {
                return next == WeightedActionItemPreferenceStatus::Ready;
            }
            WeightedActionItemPreferenceStatus::Ready => {
                return next == WeightedActionItemPreferenceStatus::Stopped;
            }
            WeightedActionItemPreferenceStatus::Draining => {
                match next {
                    WeightedActionItemPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedActionItemPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted action item preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ceiling - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedActionItemPreference {
        WeightedActionItemPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            ceiling: self.ceiling,
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            visible: self.visible,
            segment: self.segment.clone(),
            priority: self.priority,
            title: self.title.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted action item preference in status idle.
pub fn new_weighted_action_item_preference(code: &str, name: &str) -> WeightedActionItemPreference {
    WeightedActionItemPreference {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        ceiling: 0.0,
        minimum: 0.0,
        attributes: BTreeMap::new(),
        visible: false,
        segment: "".to_string(),
        priority: 0,
        title: "".to_string(),
        principal: "".to_string(),
        status: WeightedActionItemPreferenceStatus::Idle,
    }
}
