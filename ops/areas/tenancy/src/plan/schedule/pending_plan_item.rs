use super::*;

use acme_kit::text;

/// Where a pending plan item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPlanItemStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingPlanItemStatus {
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

impl std::fmt::Display for PendingPlanItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending plan item as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanItem {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub attempts: i64,
    pub maximum: f64,
    pub principal: String,
    pub amount: f64,
    pub ratio: f64,
    pub scope: String,
    pub tags: Vec<String>,
    pub label: String,
    pub status: PendingPlanItemStatus,
}

impl PendingPlanItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPlanItemStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPlanItemStatus) -> bool {
        match self.status {
            PendingPlanItemStatus::Idle => {
                return next == PendingPlanItemStatus::Stopped;
            }
            PendingPlanItemStatus::Warming => {
                return next == PendingPlanItemStatus::Draining;
            }
            PendingPlanItemStatus::Ready => {
                return next == PendingPlanItemStatus::Draining || next == PendingPlanItemStatus::Stopped;
            }
            PendingPlanItemStatus::Draining => {
                return next == PendingPlanItemStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPlanItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending plan item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending plan item", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPlanItem {
        PendingPlanItem {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            attempts: self.attempts,
            maximum: self.maximum,
            principal: self.principal.clone(),
            amount: self.amount,
            ratio: self.ratio,
            scope: self.scope.clone(),
            tags: self.tags.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending plan item in status idle.
pub fn new_pending_plan_item(code: &str, title: &str) -> PendingPlanItem {
    PendingPlanItem {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        attempts: 0,
        maximum: 0.0,
        principal: "".to_string(),
        amount: 0.0,
        ratio: 0.0,
        scope: "".to_string(),
        tags: Vec::new(),
        label: "".to_string(),
        status: PendingPlanItemStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_plan_item("code-695", "title-701");
        assert_eq!(item.status, PendingPlanItemStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_pending_plan_item("code-695", "title-701");
        if item.move_to(PendingPlanItemStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPlanItemStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_plan_item("code-695", "title-701");
        item.status = PendingPlanItemStatus::Stopped;
        assert!(
            item.move_to(PendingPlanItemStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
