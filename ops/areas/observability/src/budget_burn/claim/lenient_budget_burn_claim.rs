use super::*;

use acme_kit::text;

/// Where a lenient budget burn claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientBudgetBurnClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientBudgetBurnClaimStatus {
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

impl std::fmt::Display for LenientBudgetBurnClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient budget burn claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBudgetBurnClaim {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub namespace: String,
    pub channel: String,
    pub starts_at: i64,
    pub key_id: String,
    pub score: f64,
    pub quota_units: i64,
    pub title: String,
    pub status: LenientBudgetBurnClaimStatus,
}

impl LenientBudgetBurnClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientBudgetBurnClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientBudgetBurnClaimStatus) -> bool {
        match self.status {
            LenientBudgetBurnClaimStatus::Idle => {
                match next {
                    LenientBudgetBurnClaimStatus::Warming | LenientBudgetBurnClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBudgetBurnClaimStatus::Warming => {
                match next {
                    LenientBudgetBurnClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBudgetBurnClaimStatus::Ready => {
                return next == LenientBudgetBurnClaimStatus::Draining || next == LenientBudgetBurnClaimStatus::Stopped;
            }
            LenientBudgetBurnClaimStatus::Draining => {
                match next {
                    LenientBudgetBurnClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientBudgetBurnClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient budget burn claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.starts_at as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientBudgetBurnClaim {
        LenientBudgetBurnClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            namespace: self.namespace.clone(),
            channel: self.channel.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            score: self.score,
            quota_units: self.quota_units,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient budget burn claim in status idle.
pub fn new_lenient_budget_burn_claim(code: &str, label: &str) -> LenientBudgetBurnClaim {
    LenientBudgetBurnClaim {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        namespace: "".to_string(),
        channel: "".to_string(),
        starts_at: 0,
        key_id: "".to_string(),
        score: 0.0,
        quota_units: 0,
        title: "".to_string(),
        status: LenientBudgetBurnClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_budget_burn_claim("code-525", "label-459");
        assert_eq!(item.status, LenientBudgetBurnClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_budget_burn_claim("code-525", "label-459");
        if item.move_to(LenientBudgetBurnClaimStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientBudgetBurnClaimStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_budget_burn_claim("code-525", "label-459");
        item.status = LenientBudgetBurnClaimStatus::Stopped;
        assert!(
            item.move_to(LenientBudgetBurnClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
