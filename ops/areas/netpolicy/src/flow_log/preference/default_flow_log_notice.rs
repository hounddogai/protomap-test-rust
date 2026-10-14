use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default flow log notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFlowLogNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultFlowLogNoticeStatus {
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

impl std::fmt::Display for DefaultFlowLogNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default flow log notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogNotice {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub tags: Vec<String>,
    pub scope: String,
    pub score: f64,
    pub status: DefaultFlowLogNoticeStatus,
}

impl DefaultFlowLogNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFlowLogNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFlowLogNoticeStatus) -> bool {
        match self.status {
            DefaultFlowLogNoticeStatus::Planned => {
                match next {
                    DefaultFlowLogNoticeStatus::InProgress | DefaultFlowLogNoticeStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogNoticeStatus::Scheduled => {
                match next {
                    DefaultFlowLogNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogNoticeStatus::InProgress => {
                return next == DefaultFlowLogNoticeStatus::Completed || next == DefaultFlowLogNoticeStatus::Cancelled;
            }
            DefaultFlowLogNoticeStatus::Completed => {
                match next {
                    DefaultFlowLogNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultFlowLogNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default flow log notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn copy(&self) -> DefaultFlowLogNotice {
        DefaultFlowLogNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            tags: self.tags.clone(),
            scope: self.scope.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default flow log notice in status planned.
pub fn new_default_flow_log_notice(code: &str, title: &str) -> DefaultFlowLogNotice {
    DefaultFlowLogNotice {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        tags: Vec::new(),
        scope: "".to_string(),
        score: 0.0,
        status: DefaultFlowLogNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_flow_log_notice("code-813", "title-311");
        assert_eq!(item.status, DefaultFlowLogNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_flow_log_notice("code-813", "title-311");
        if item.move_to(DefaultFlowLogNoticeStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFlowLogNoticeStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_flow_log_notice("code-813", "title-311");
        item.status = DefaultFlowLogNoticeStatus::Cancelled;
        assert!(
            item.move_to(DefaultFlowLogNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
