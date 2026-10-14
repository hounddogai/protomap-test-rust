use super::*;

use acme_kit::text;

/// Where a strict usage plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictUsagePlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictUsagePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictUsagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict usage plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictUsagePlan {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub minimum: f64,
    pub tags: Vec<String>,
    pub score: f64,
    pub status: StrictUsagePlanStatus,
}

impl StrictUsagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictUsagePlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictUsagePlanStatus) -> bool {
        match self.status {
            StrictUsagePlanStatus::Requested => {
                match next {
                    StrictUsagePlanStatus::Revoked | StrictUsagePlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictUsagePlanStatus::Granted => {
                return next == StrictUsagePlanStatus::Expired;
            }
            StrictUsagePlanStatus::Revoked => {
                return next == StrictUsagePlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictUsagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict usage plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.minimum + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictUsagePlan {
        StrictUsagePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            tags: self.tags.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict usage plan in status requested.
pub fn new_strict_usage_plan(code: &str, name: &str) -> StrictUsagePlan {
    StrictUsagePlan {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        minimum: 0.0,
        tags: Vec::new(),
        score: 0.0,
        status: StrictUsagePlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_usage_plan("code-31", "name-22");
        assert_eq!(item.status, StrictUsagePlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_strict_usage_plan("code-31", "name-22");
        if item.move_to(StrictUsagePlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictUsagePlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_usage_plan("code-31", "name-22");
        item.status = StrictUsagePlanStatus::Expired;
        assert!(
            item.move_to(StrictUsagePlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
