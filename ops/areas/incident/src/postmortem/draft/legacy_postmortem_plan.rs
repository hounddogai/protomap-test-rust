use super::*;

use acme_kit::text;

/// Where a legacy postmortem plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPostmortemPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyPostmortemPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyPostmortemPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy postmortem plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemPlan {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub segment: String,
    pub owner_id: String,
    pub cost: f64,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub floor_value: f64,
    pub status: LegacyPostmortemPlanStatus,
}

impl LegacyPostmortemPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPostmortemPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPostmortemPlanStatus) -> bool {
        match self.status {
            LegacyPostmortemPlanStatus::Created => {
                return next == LegacyPostmortemPlanStatus::Retired;
            }
            LegacyPostmortemPlanStatus::Validated => {
                return next == LegacyPostmortemPlanStatus::Published;
            }
            LegacyPostmortemPlanStatus::Published => {
                return next == LegacyPostmortemPlanStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPostmortemPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy postmortem plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy postmortem plan", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        (self.floor_value * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPostmortemPlan {
        LegacyPostmortemPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            segment: self.segment.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy postmortem plan in status created.
pub fn new_legacy_postmortem_plan(code: &str, title: &str) -> LegacyPostmortemPlan {
    LegacyPostmortemPlan {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        segment: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        tags: Vec::new(),
        fingerprint: "".to_string(),
        floor_value: 0.0,
        status: LegacyPostmortemPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_postmortem_plan("code-249", "title-900");
        assert_eq!(item.status, LegacyPostmortemPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_legacy_postmortem_plan("code-249", "title-900");
        if item.move_to(LegacyPostmortemPlanStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPostmortemPlanStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_postmortem_plan("code-249", "title-900");
        item.status = LegacyPostmortemPlanStatus::Retired;
        assert!(
            item.move_to(LegacyPostmortemPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
