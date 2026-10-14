use super::*;

use acme_kit::text;

/// Where a page plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PagePlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PagePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PagePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PagePlan {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub amount: f64,
    pub quantity: i64,
    pub grace_days: i64,
    pub status: PagePlanStatus,
}

impl PagePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PagePlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PagePlanStatus) -> bool {
        match self.status {
            PagePlanStatus::New => {
                match next {
                    PagePlanStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePlanStatus::Queued => {
                match next {
                    PagePlanStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePlanStatus::Running => {
                match next {
                    PagePlanStatus::Done | PagePlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePlanStatus::Done => {
                match next {
                    PagePlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: PagePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "page plan", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PagePlan {
        PagePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            quantity: self.quantity,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page plan in status new.
pub fn new_page_plan(code: &str, label: &str) -> PagePlan {
    PagePlan {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        amount: 0.0,
        quantity: 0,
        grace_days: 0,
        status: PagePlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_page_plan("code-23", "label-899");
        assert_eq!(item.status, PagePlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_page_plan("code-23", "label-899");
        if item.move_to(PagePlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PagePlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_page_plan("code-23", "label-899");
        item.status = PagePlanStatus::Failed;
        assert!(item.move_to(PagePlanStatus::New).is_err(), "expected move to fail");
    }
}
