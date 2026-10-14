use super::*;

use acme_kit::text;

/// Where a tiered plan item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPlanItemStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredPlanItemStatus {
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

impl std::fmt::Display for TieredPlanItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered plan item as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItem {
    pub code: String,
    pub title: String,
    pub source: String,
    pub version: i64,
    pub host: String,
    pub status: TieredPlanItemStatus,
}

impl TieredPlanItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPlanItemStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPlanItemStatus) -> bool {
        match self.status {
            TieredPlanItemStatus::Idle => {
                match next {
                    TieredPlanItemStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPlanItemStatus::Warming => {
                return next == TieredPlanItemStatus::Stopped;
            }
            TieredPlanItemStatus::Ready => {
                return next == TieredPlanItemStatus::Draining || next == TieredPlanItemStatus::Stopped;
            }
            TieredPlanItemStatus::Draining => {
                return next == TieredPlanItemStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPlanItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered plan item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> TieredPlanItem {
        TieredPlanItem {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            version: self.version,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered plan item in status idle.
pub fn new_tiered_plan_item(code: &str, title: &str) -> TieredPlanItem {
    TieredPlanItem {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        version: 0,
        host: "".to_string(),
        status: TieredPlanItemStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_plan_item("code-695", "title-796");
        assert_eq!(item.status, TieredPlanItemStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_plan_item("code-695", "title-796");
        if item.move_to(TieredPlanItemStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPlanItemStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_plan_item("code-695", "title-796");
        item.status = TieredPlanItemStatus::Stopped;
        assert!(
            item.move_to(TieredPlanItemStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
