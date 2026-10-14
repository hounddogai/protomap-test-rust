use super::*;

use acme_kit::text;

/// Where a plan item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanItemStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PlanItemStatus {
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

impl std::fmt::Display for PlanItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanItem {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub ratio: f64,
    pub notes: String,
    pub revision: i64,
    pub attempts: i64,
    pub status: PlanItemStatus,
}

impl PlanItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanItemStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanItemStatus) -> bool {
        match self.status {
            PlanItemStatus::Draft => {
                match next {
                    PlanItemStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanItemStatus::Pending => {
                return next == PlanItemStatus::Active || next == PlanItemStatus::Closed;
            }
            PlanItemStatus::Active => {
                match next {
                    PlanItemStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanItemStatus::Suspended => {
                return next == PlanItemStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanItem {
        PlanItem {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            revision: self.revision,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan item in status draft.
pub fn new_plan_item(code: &str, name: &str) -> PlanItem {
    PlanItem {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        revision: 0,
        attempts: 0,
        status: PlanItemStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_plan_item("code-501", "name-223");
        assert_eq!(item.status, PlanItemStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_plan_item("code-501", "name-223");
        if item.move_to(PlanItemStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanItemStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_plan_item("code-501", "name-223");
        item.status = PlanItemStatus::Closed;
        assert!(item.move_to(PlanItemStatus::Draft).is_err(), "expected move to fail");
    }
}
