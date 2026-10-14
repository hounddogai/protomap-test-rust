use super::*;

use acme_kit::text;

/// Where a plan threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanThresholdStatus {
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

impl std::fmt::Display for PlanThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanThreshold {
    pub code: String,
    pub title: String,
    pub source: String,
    pub ends_at: i64,
    pub description: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub phone: String,
    pub maximum: f64,
    pub principal: String,
    pub status: PlanThresholdStatus,
}

impl PlanThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanThresholdStatus) -> bool {
        match self.status {
            PlanThresholdStatus::Idle => {
                match next {
                    PlanThresholdStatus::Warming | PlanThresholdStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanThresholdStatus::Warming => {
                return next == PlanThresholdStatus::Ready;
            }
            PlanThresholdStatus::Ready => {
                return next == PlanThresholdStatus::Draining || next == PlanThresholdStatus::Stopped;
            }
            PlanThresholdStatus::Draining => {
                match next {
                    PlanThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan threshold", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanThreshold {
        PlanThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            ends_at: self.ends_at,
            description: self.description.clone(),
            tags: self.tags.clone(),
            version: self.version,
            phone: self.phone.clone(),
            maximum: self.maximum,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a plan threshold in status idle.
pub fn new_plan_threshold(code: &str, title: &str) -> PlanThreshold {
    PlanThreshold {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        ends_at: 0,
        description: "".to_string(),
        tags: Vec::new(),
        version: 0,
        phone: "".to_string(),
        maximum: 0.0,
        principal: "".to_string(),
        status: PlanThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_plan_threshold("code-509", "title-96");
        assert_eq!(item.status, PlanThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_plan_threshold("code-509", "title-96");
        if item.move_to(PlanThresholdStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanThresholdStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_plan_threshold("code-509", "title-96");
        item.status = PlanThresholdStatus::Stopped;
        assert!(
            item.move_to(PlanThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
