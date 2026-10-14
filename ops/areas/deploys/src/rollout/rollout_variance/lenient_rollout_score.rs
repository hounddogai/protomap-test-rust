use super::*;

use acme_kit::text;

/// Where a lenient rollout score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRolloutScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientRolloutScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientRolloutScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient rollout score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutScore {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub quantity: i64,
    pub key_id: String,
    pub status: LenientRolloutScoreStatus,
}

impl LenientRolloutScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRolloutScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRolloutScoreStatus) -> bool {
        match self.status {
            LenientRolloutScoreStatus::Created => {
                match next {
                    LenientRolloutScoreStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRolloutScoreStatus::Validated => {
                match next {
                    LenientRolloutScoreStatus::Published | LenientRolloutScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRolloutScoreStatus::Published => {
                match next {
                    LenientRolloutScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientRolloutScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rollout score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRolloutScore {
        LenientRolloutScore {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient rollout score in status created.
pub fn new_lenient_rollout_score(code: &str, title: &str) -> LenientRolloutScore {
    LenientRolloutScore {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        quantity: 0,
        key_id: "".to_string(),
        status: LenientRolloutScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_rollout_score("code-576", "title-829");
        assert_eq!(item.status, LenientRolloutScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_rollout_score("code-576", "title-829");
        if item.move_to(LenientRolloutScoreStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRolloutScoreStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_rollout_score("code-576", "title-829");
        item.status = LenientRolloutScoreStatus::Retired;
        assert!(
            item.move_to(LenientRolloutScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
