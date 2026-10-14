use super::*;

use acme_kit::text;

/// Where a rolling postmortem score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPostmortemScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingPostmortemScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingPostmortemScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling postmortem score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPostmortemScore {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub priority: i64,
    pub revision: i64,
    pub tags: Vec<String>,
    pub namespace: String,
    pub cost: f64,
    pub description: String,
    pub status: RollingPostmortemScoreStatus,
}

impl RollingPostmortemScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPostmortemScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPostmortemScoreStatus) -> bool {
        match self.status {
            RollingPostmortemScoreStatus::Draft => {
                match next {
                    RollingPostmortemScoreStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemScoreStatus::Pending => {
                match next {
                    RollingPostmortemScoreStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPostmortemScoreStatus::Active => {
                return next == RollingPostmortemScoreStatus::Closed;
            }
            RollingPostmortemScoreStatus::Suspended => {
                return next == RollingPostmortemScoreStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPostmortemScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling postmortem score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPostmortemScore {
        RollingPostmortemScore {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            priority: self.priority,
            revision: self.revision,
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            cost: self.cost,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling postmortem score in status draft.
pub fn new_rolling_postmortem_score(code: &str, name: &str) -> RollingPostmortemScore {
    RollingPostmortemScore {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        priority: 0,
        revision: 0,
        tags: Vec::new(),
        namespace: "".to_string(),
        cost: 0.0,
        description: "".to_string(),
        status: RollingPostmortemScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_postmortem_score("code-433", "name-189");
        assert_eq!(item.status, RollingPostmortemScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_postmortem_score("code-433", "name-189");
        if item.move_to(RollingPostmortemScoreStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPostmortemScoreStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_postmortem_score("code-433", "name-189");
        item.status = RollingPostmortemScoreStatus::Closed;
        assert!(
            item.move_to(RollingPostmortemScoreStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
