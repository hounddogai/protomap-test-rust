use super::*;

use acme_kit::text;

/// Where a fallback tenant rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTenantRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackTenantRuleStatus {
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

impl std::fmt::Display for FallbackTenantRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback tenant rule as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantRule {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub channel: String,
    pub batch_size: i64,
    pub external_ref: String,
    pub status: FallbackTenantRuleStatus,
}

impl FallbackTenantRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTenantRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTenantRuleStatus) -> bool {
        match self.status {
            FallbackTenantRuleStatus::Idle => {
                match next {
                    FallbackTenantRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantRuleStatus::Warming => {
                return next == FallbackTenantRuleStatus::Draining;
            }
            FallbackTenantRuleStatus::Ready => {
                match next {
                    FallbackTenantRuleStatus::Draining | FallbackTenantRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantRuleStatus::Draining => {
                match next {
                    FallbackTenantRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackTenantRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback tenant rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback tenant rule", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTenantRule {
        FallbackTenantRule {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            channel: self.channel.clone(),
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback tenant rule in status idle.
pub fn new_fallback_tenant_rule(code: &str, title: &str) -> FallbackTenantRule {
    FallbackTenantRule {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        channel: "".to_string(),
        batch_size: 0,
        external_ref: "".to_string(),
        status: FallbackTenantRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_tenant_rule("code-142", "title-734");
        assert_eq!(item.status, FallbackTenantRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_fallback_tenant_rule("code-142", "title-734");
        if item.move_to(FallbackTenantRuleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTenantRuleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_tenant_rule("code-142", "title-734");
        item.status = FallbackTenantRuleStatus::Stopped;
        assert!(
            item.move_to(FallbackTenantRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
