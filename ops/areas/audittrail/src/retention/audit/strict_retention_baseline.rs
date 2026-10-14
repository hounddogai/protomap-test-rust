use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict retention baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRetentionBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictRetentionBaselineStatus {
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

impl std::fmt::Display for StrictRetentionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict retention baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionBaseline {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub expires_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub country: String,
    pub budget: f64,
    pub status: StrictRetentionBaselineStatus,
}

impl StrictRetentionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRetentionBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRetentionBaselineStatus) -> bool {
        match self.status {
            StrictRetentionBaselineStatus::Planned => {
                return next == StrictRetentionBaselineStatus::Scheduled;
            }
            StrictRetentionBaselineStatus::Scheduled => {
                match next {
                    StrictRetentionBaselineStatus::InProgress | StrictRetentionBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRetentionBaselineStatus::InProgress => {
                match next {
                    StrictRetentionBaselineStatus::Completed | StrictRetentionBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRetentionBaselineStatus::Completed => {
                match next {
                    StrictRetentionBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: StrictRetentionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict retention baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRetentionBaseline {
        StrictRetentionBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            expires_at: self.expires_at,
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict retention baseline in status planned.
pub fn new_strict_retention_baseline(code: &str, title: &str) -> StrictRetentionBaseline {
    StrictRetentionBaseline {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        expires_at: 0,
        attributes: BTreeMap::new(),
        name: "".to_string(),
        country: "".to_string(),
        budget: 0.0,
        status: StrictRetentionBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_retention_baseline("code-683", "title-862");
        assert_eq!(item.status, StrictRetentionBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_strict_retention_baseline("code-683", "title-862");
        if item.move_to(StrictRetentionBaselineStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRetentionBaselineStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_retention_baseline("code-683", "title-862");
        item.status = StrictRetentionBaselineStatus::Cancelled;
        assert!(
            item.move_to(StrictRetentionBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
