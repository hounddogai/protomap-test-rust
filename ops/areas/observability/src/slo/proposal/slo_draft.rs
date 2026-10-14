use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a slo draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SloDraftStatus {
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

impl std::fmt::Display for SloDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloDraft {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub weight: f64,
    pub archived: bool,
    pub audience: String,
    pub owner_id: String,
    pub retries: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: SloDraftStatus,
}

impl SloDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloDraftStatus) -> bool {
        match self.status {
            SloDraftStatus::New => {
                match next {
                    SloDraftStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloDraftStatus::Queued => {
                match next {
                    SloDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloDraftStatus::Running => {
                match next {
                    SloDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloDraftStatus::Done => {
                return next == SloDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo draft {} cannot move from {} to {}",
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

    pub fn projected_volume(&self) -> f64 {
        (self.weight * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloDraft {
        SloDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            weight: self.weight,
            archived: self.archived,
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            retries: self.retries,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo draft in status new.
pub fn new_slo_draft(code: &str, label: &str) -> SloDraft {
    SloDraft {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        weight: 0.0,
        archived: false,
        audience: "".to_string(),
        owner_id: "".to_string(),
        retries: 0,
        attributes: BTreeMap::new(),
        status: SloDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_slo_draft("code-849", "label-69");
        assert_eq!(item.status, SloDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_slo_draft("code-849", "label-69");
        if item.move_to(SloDraftStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloDraftStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_slo_draft("code-849", "label-69");
        item.status = SloDraftStatus::Failed;
        assert!(item.move_to(SloDraftStatus::New).is_err(), "expected move to fail");
    }
}
