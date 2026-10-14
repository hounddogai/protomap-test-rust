use super::*;

use acme_kit::text;

/// Where a isolation plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationPlanStatus {
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

impl std::fmt::Display for IsolationPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPlan {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub visible: bool,
    pub discount: f64,
    pub name: String,
    pub source: String,
    pub segment: String,
    pub status: IsolationPlanStatus,
}

impl IsolationPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationPlanStatus) -> bool {
        match self.status {
            IsolationPlanStatus::Idle => {
                return next == IsolationPlanStatus::Stopped;
            }
            IsolationPlanStatus::Warming => {
                return next == IsolationPlanStatus::Ready || next == IsolationPlanStatus::Draining;
            }
            IsolationPlanStatus::Ready => {
                return next == IsolationPlanStatus::Draining || next == IsolationPlanStatus::Stopped;
            }
            IsolationPlanStatus::Draining => {
                return next == IsolationPlanStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> IsolationPlan {
        IsolationPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            visible: self.visible,
            discount: self.discount,
            name: self.name.clone(),
            source: self.source.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation plan in status idle.
pub fn new_isolation_plan(code: &str, title: &str) -> IsolationPlan {
    IsolationPlan {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        visible: false,
        discount: 0.0,
        name: "".to_string(),
        source: "".to_string(),
        segment: "".to_string(),
        status: IsolationPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_plan("code-28", "title-537");
        assert_eq!(item.status, IsolationPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_isolation_plan("code-28", "title-537");
        if item.move_to(IsolationPlanStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationPlanStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_plan("code-28", "title-537");
        item.status = IsolationPlanStatus::Stopped;
        assert!(
            item.move_to(IsolationPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
