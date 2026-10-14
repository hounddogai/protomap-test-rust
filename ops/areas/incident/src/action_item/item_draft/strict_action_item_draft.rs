use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict action item draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictActionItemDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictActionItemDraftStatus {
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

impl std::fmt::Display for StrictActionItemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict action item draft as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictActionItemDraft {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub namespace: String,
    pub status: StrictActionItemDraftStatus,
}

impl StrictActionItemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictActionItemDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictActionItemDraftStatus) -> bool {
        match self.status {
            StrictActionItemDraftStatus::Idle => {
                return next == StrictActionItemDraftStatus::Draining || next == StrictActionItemDraftStatus::Stopped;
            }
            StrictActionItemDraftStatus::Warming => {
                match next {
                    StrictActionItemDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictActionItemDraftStatus::Ready => {
                return next == StrictActionItemDraftStatus::Draining || next == StrictActionItemDraftStatus::Stopped;
            }
            StrictActionItemDraftStatus::Draining => {
                return next == StrictActionItemDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictActionItemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict action item draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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
        self.updated_at as f64
    }

    pub fn copy(&self) -> StrictActionItemDraft {
        StrictActionItemDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict action item draft in status idle.
pub fn new_strict_action_item_draft(code: &str, title: &str) -> StrictActionItemDraft {
    StrictActionItemDraft {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        host: "".to_string(),
        attributes: BTreeMap::new(),
        updated_at: 0,
        namespace: "".to_string(),
        status: StrictActionItemDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_action_item_draft("code-424", "title-59");
        assert_eq!(item.status, StrictActionItemDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_strict_action_item_draft("code-424", "title-59");
        if item.move_to(StrictActionItemDraftStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictActionItemDraftStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_action_item_draft("code-424", "title-59");
        item.status = StrictActionItemDraftStatus::Stopped;
        assert!(
            item.move_to(StrictActionItemDraftStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
