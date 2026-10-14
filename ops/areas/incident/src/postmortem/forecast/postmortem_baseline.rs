use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a postmortem baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemBaselineStatus {
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

impl std::fmt::Display for PostmortemBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaseline {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub scope: String,
    pub batch_size: i64,
    pub status: PostmortemBaselineStatus,
}

impl PostmortemBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemBaselineStatus) -> bool {
        match self.status {
            PostmortemBaselineStatus::Idle => {
                return next == PostmortemBaselineStatus::Ready;
            }
            PostmortemBaselineStatus::Warming => {
                match next {
                    PostmortemBaselineStatus::Ready | PostmortemBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBaselineStatus::Ready => {
                match next {
                    PostmortemBaselineStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBaselineStatus::Draining => {
                return next == PostmortemBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn effective_value(&self) -> f64 {
        ((self.threshold - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemBaseline {
        PostmortemBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            scope: self.scope.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a postmortem baseline in status idle.
pub fn new_postmortem_baseline(code: &str, label: &str) -> PostmortemBaseline {
    PostmortemBaseline {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        scope: "".to_string(),
        batch_size: 0,
        status: PostmortemBaselineStatus::Idle,
    }
}
