use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a limit bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LimitBundleStatus {
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

impl std::fmt::Display for LimitBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitBundle {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub source: String,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub created_at: i64,
    pub minimum: f64,
    pub budget: f64,
    pub port: i64,
    pub status: LimitBundleStatus,
}

impl LimitBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitBundleStatus) -> bool {
        match self.status {
            LimitBundleStatus::Planned => {
                return next == LimitBundleStatus::InProgress || next == LimitBundleStatus::Completed;
            }
            LimitBundleStatus::Scheduled => {
                return next == LimitBundleStatus::InProgress;
            }
            LimitBundleStatus::InProgress => {
                return next == LimitBundleStatus::Cancelled;
            }
            LimitBundleStatus::Completed => {
                return next == LimitBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
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

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitBundle {
        LimitBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            source: self.source.clone(),
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            created_at: self.created_at,
            minimum: self.minimum,
            budget: self.budget,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a limit bundle in status planned.
pub fn new_limit_bundle(code: &str, label: &str) -> LimitBundle {
    LimitBundle {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        source: "".to_string(),
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        created_at: 0,
        minimum: 0.0,
        budget: 0.0,
        port: 0,
        status: LimitBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_limit_bundle("code-563", "label-599");
        assert_eq!(item.status, LimitBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_limit_bundle("code-563", "label-599");
        if item.move_to(LimitBundleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitBundleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_limit_bundle("code-563", "label-599");
        item.status = LimitBundleStatus::Cancelled;
        assert!(
            item.move_to(LimitBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
