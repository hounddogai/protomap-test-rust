use super::*;

use acme_kit::text;

/// Where a primary sink plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySinkPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimarySinkPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimarySinkPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary sink plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkPlan {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub country: String,
    pub batch_size: i64,
    pub rate: f64,
    pub notes: String,
    pub reviewer: String,
    pub quantity: i64,
    pub status: PrimarySinkPlanStatus,
}

impl PrimarySinkPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySinkPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySinkPlanStatus) -> bool {
        match self.status {
            PrimarySinkPlanStatus::Created => {
                return next == PrimarySinkPlanStatus::Retired;
            }
            PrimarySinkPlanStatus::Validated => {
                match next {
                    PrimarySinkPlanStatus::Published | PrimarySinkPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkPlanStatus::Published => {
                match next {
                    PrimarySinkPlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimarySinkPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary sink plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.rate * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySinkPlan {
        PrimarySinkPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            country: self.country.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            notes: self.notes.clone(),
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary sink plan in status created.
pub fn new_primary_sink_plan(code: &str, title: &str) -> PrimarySinkPlan {
    PrimarySinkPlan {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        country: "".to_string(),
        batch_size: 0,
        rate: 0.0,
        notes: "".to_string(),
        reviewer: "".to_string(),
        quantity: 0,
        status: PrimarySinkPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_sink_plan("code-843", "title-30");
        assert_eq!(item.status, PrimarySinkPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_primary_sink_plan("code-843", "title-30");
        if item.move_to(PrimarySinkPlanStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySinkPlanStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_sink_plan("code-843", "title-30");
        item.status = PrimarySinkPlanStatus::Retired;
        assert!(
            item.move_to(PrimarySinkPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
