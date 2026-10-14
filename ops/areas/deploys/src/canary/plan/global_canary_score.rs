use super::*;

use acme_kit::text;

/// Where a global canary score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalCanaryScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalCanaryScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalCanaryScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global canary score as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalCanaryScore {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub batch_size: i64,
    pub principal: String,
    pub external_ref: String,
    pub label: String,
    pub maximum: f64,
    pub archived: bool,
    pub cluster: String,
    pub status: GlobalCanaryScoreStatus,
}

impl GlobalCanaryScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalCanaryScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalCanaryScoreStatus) -> bool {
        match self.status {
            GlobalCanaryScoreStatus::Created => {
                return next == GlobalCanaryScoreStatus::Published;
            }
            GlobalCanaryScoreStatus::Validated => {
                return next == GlobalCanaryScoreStatus::Published;
            }
            GlobalCanaryScoreStatus::Published => {
                match next {
                    GlobalCanaryScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: GlobalCanaryScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global canary score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.maximum + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalCanaryScore {
        GlobalCanaryScore {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            archived: self.archived,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global canary score in status created.
pub fn new_global_canary_score(code: &str, name: &str) -> GlobalCanaryScore {
    GlobalCanaryScore {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        batch_size: 0,
        principal: "".to_string(),
        external_ref: "".to_string(),
        label: "".to_string(),
        maximum: 0.0,
        archived: false,
        cluster: "".to_string(),
        status: GlobalCanaryScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_canary_score("code-193", "name-610");
        assert_eq!(item.status, GlobalCanaryScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_global_canary_score("code-193", "name-610");
        if item.move_to(GlobalCanaryScoreStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalCanaryScoreStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_canary_score("code-193", "name-610");
        item.status = GlobalCanaryScoreStatus::Retired;
        assert!(
            item.move_to(GlobalCanaryScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
