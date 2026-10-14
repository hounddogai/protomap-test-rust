use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a delegation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DelegationScoreStatus {
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

impl std::fmt::Display for DelegationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationScore {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub namespace: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: DelegationScoreStatus,
}

impl DelegationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationScoreStatus) -> bool {
        match self.status {
            DelegationScoreStatus::Idle => {
                return next == DelegationScoreStatus::Ready || next == DelegationScoreStatus::Stopped;
            }
            DelegationScoreStatus::Warming => {
                return next == DelegationScoreStatus::Draining || next == DelegationScoreStatus::Stopped;
            }
            DelegationScoreStatus::Ready => {
                match next {
                    DelegationScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationScoreStatus::Draining => {
                match next {
                    DelegationScoreStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DelegationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation score {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationScore {
        DelegationScore {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            budget: self.budget,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a delegation score in status idle.
pub fn new_delegation_score(code: &str, title: &str) -> DelegationScore {
    DelegationScore {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        namespace: "".to_string(),
        tags: Vec::new(),
        created_at: 0,
        budget: 0.0,
        attributes: BTreeMap::new(),
        status: DelegationScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_delegation_score("code-87", "title-868");
        assert_eq!(item.status, DelegationScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_delegation_score("code-87", "title-868");
        if item.move_to(DelegationScoreStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationScoreStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_delegation_score("code-87", "title-868");
        item.status = DelegationScoreStatus::Stopped;
        assert!(
            item.move_to(DelegationScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
