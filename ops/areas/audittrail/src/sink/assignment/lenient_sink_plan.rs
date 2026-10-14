use super::*;

use acme_kit::text;

/// Where a lenient sink plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSinkPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientSinkPlanStatus {
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

impl std::fmt::Display for LenientSinkPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient sink plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkPlan {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub created_at: i64,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub issuer: String,
    pub status: LenientSinkPlanStatus,
}

impl LenientSinkPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSinkPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSinkPlanStatus) -> bool {
        match self.status {
            LenientSinkPlanStatus::Draft => {
                return next == LenientSinkPlanStatus::Active || next == LenientSinkPlanStatus::Suspended;
            }
            LenientSinkPlanStatus::Pending => {
                match next {
                    LenientSinkPlanStatus::Active | LenientSinkPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSinkPlanStatus::Active => {
                match next {
                    LenientSinkPlanStatus::Suspended | LenientSinkPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSinkPlanStatus::Suspended => {
                match next {
                    LenientSinkPlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientSinkPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient sink plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient sink plan", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSinkPlan {
        LenientSinkPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            created_at: self.created_at,
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient sink plan in status draft.
pub fn new_lenient_sink_plan(code: &str, name: &str) -> LenientSinkPlan {
    LenientSinkPlan {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        created_at: 0,
        revision: 0,
        ttl_seconds: 0,
        external_ref: "".to_string(),
        issuer: "".to_string(),
        status: LenientSinkPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_sink_plan("code-946", "name-548");
        assert_eq!(item.status, LenientSinkPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_lenient_sink_plan("code-946", "name-548");
        if item.move_to(LenientSinkPlanStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSinkPlanStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_sink_plan("code-946", "name-548");
        item.status = LenientSinkPlanStatus::Closed;
        assert!(
            item.move_to(LenientSinkPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
