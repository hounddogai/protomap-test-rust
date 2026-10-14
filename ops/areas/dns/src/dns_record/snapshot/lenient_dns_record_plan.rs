use super::*;

use acme_kit::text;

/// Where a lenient dns record plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDnsRecordPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientDnsRecordPlanStatus {
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

impl std::fmt::Display for LenientDnsRecordPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient dns record plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordPlan {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub maximum: f64,
    pub pinned: bool,
    pub quantity: i64,
    pub summary: String,
    pub tenant_id: String,
    pub revision: i64,
    pub locale: String,
    pub status: LenientDnsRecordPlanStatus,
}

impl LenientDnsRecordPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDnsRecordPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDnsRecordPlanStatus) -> bool {
        match self.status {
            LenientDnsRecordPlanStatus::Idle => {
                return next == LenientDnsRecordPlanStatus::Warming || next == LenientDnsRecordPlanStatus::Stopped;
            }
            LenientDnsRecordPlanStatus::Warming => {
                match next {
                    LenientDnsRecordPlanStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDnsRecordPlanStatus::Ready => {
                match next {
                    LenientDnsRecordPlanStatus::Draining | LenientDnsRecordPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDnsRecordPlanStatus::Draining => {
                return next == LenientDnsRecordPlanStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDnsRecordPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dns record plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient dns record plan", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.revision as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDnsRecordPlan {
        LenientDnsRecordPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            maximum: self.maximum,
            pinned: self.pinned,
            quantity: self.quantity,
            summary: self.summary.clone(),
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient dns record plan in status idle.
pub fn new_lenient_dns_record_plan(code: &str, name: &str) -> LenientDnsRecordPlan {
    LenientDnsRecordPlan {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        maximum: 0.0,
        pinned: false,
        quantity: 0,
        summary: "".to_string(),
        tenant_id: "".to_string(),
        revision: 0,
        locale: "".to_string(),
        status: LenientDnsRecordPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_dns_record_plan("code-292", "name-198");
        assert_eq!(item.status, LenientDnsRecordPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_dns_record_plan("code-292", "name-198");
        if item.move_to(LenientDnsRecordPlanStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDnsRecordPlanStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_dns_record_plan("code-292", "name-198");
        item.status = LenientDnsRecordPlanStatus::Stopped;
        assert!(
            item.move_to(LenientDnsRecordPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
