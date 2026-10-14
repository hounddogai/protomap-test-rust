use super::*;

use acme_kit::text;

/// Where a plan mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanMappingStatus {
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

impl std::fmt::Display for PlanMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMapping {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub region: String,
    pub sequence: i64,
    pub batch_size: i64,
    pub weight: f64,
    pub status: PlanMappingStatus,
}

impl PlanMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanMappingStatus) -> bool {
        match self.status {
            PlanMappingStatus::Idle => {
                return next == PlanMappingStatus::Stopped;
            }
            PlanMappingStatus::Warming => {
                match next {
                    PlanMappingStatus::Ready | PlanMappingStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanMappingStatus::Ready => {
                match next {
                    PlanMappingStatus::Draining | PlanMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanMappingStatus::Draining => {
                return next == PlanMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan mapping", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.sequence as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanMapping {
        PlanMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            region: self.region.clone(),
            sequence: self.sequence,
            batch_size: self.batch_size,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a plan mapping in status idle.
pub fn new_plan_mapping(code: &str, name: &str) -> PlanMapping {
    PlanMapping {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        region: "".to_string(),
        sequence: 0,
        batch_size: 0,
        weight: 0.0,
        status: PlanMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_plan_mapping("code-589", "name-418");
        assert_eq!(item.status, PlanMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_plan_mapping("code-589", "name-418");
        if item.move_to(PlanMappingStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanMappingStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_plan_mapping("code-589", "name-418");
        item.status = PlanMappingStatus::Stopped;
        assert!(item.move_to(PlanMappingStatus::Idle).is_err(), "expected move to fail");
    }
}
