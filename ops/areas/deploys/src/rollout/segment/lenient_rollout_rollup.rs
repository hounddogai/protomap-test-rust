use super::*;

use acme_kit::text;

/// Where a lenient rollout rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRolloutRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientRolloutRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientRolloutRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient rollout rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRolloutRollup {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub principal: String,
    pub approved: bool,
    pub status: LenientRolloutRollupStatus,
}

impl LenientRolloutRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRolloutRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRolloutRollupStatus) -> bool {
        match self.status {
            LenientRolloutRollupStatus::Created => {
                return next == LenientRolloutRollupStatus::Published;
            }
            LenientRolloutRollupStatus::Validated => {
                return next == LenientRolloutRollupStatus::Published || next == LenientRolloutRollupStatus::Retired;
            }
            LenientRolloutRollupStatus::Published => {
                return next == LenientRolloutRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRolloutRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rollout rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> LenientRolloutRollup {
        LenientRolloutRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            principal: self.principal.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient rollout rollup in status created.
pub fn new_lenient_rollout_rollup(code: &str, title: &str) -> LenientRolloutRollup {
    LenientRolloutRollup {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        principal: "".to_string(),
        approved: false,
        status: LenientRolloutRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_rollout_rollup("code-13", "title-997");
        assert_eq!(item.status, LenientRolloutRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_lenient_rollout_rollup("code-13", "title-997");
        if item.move_to(LenientRolloutRollupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRolloutRollupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_rollout_rollup("code-13", "title-997");
        item.status = LenientRolloutRollupStatus::Retired;
        assert!(
            item.move_to(LenientRolloutRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
