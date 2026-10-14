use super::*;

use acme_kit::text;

/// Where a tiered trust domain estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTrustDomainEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredTrustDomainEstimateStatus {
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

impl std::fmt::Display for TieredTrustDomainEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered trust domain estimate as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainEstimate {
    pub code: String,
    pub label: String,
    pub country: String,
    pub batch_size: i64,
    pub visible: bool,
    pub priority: i64,
    pub cluster: String,
    pub status: TieredTrustDomainEstimateStatus,
}

impl TieredTrustDomainEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTrustDomainEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTrustDomainEstimateStatus) -> bool {
        match self.status {
            TieredTrustDomainEstimateStatus::Idle => {
                return next == TieredTrustDomainEstimateStatus::Warming
                    || next == TieredTrustDomainEstimateStatus::Stopped;
            }
            TieredTrustDomainEstimateStatus::Warming => {
                return next == TieredTrustDomainEstimateStatus::Draining;
            }
            TieredTrustDomainEstimateStatus::Ready => {
                match next {
                    TieredTrustDomainEstimateStatus::Draining | TieredTrustDomainEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredTrustDomainEstimateStatus::Draining => {
                match next {
                    TieredTrustDomainEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredTrustDomainEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered trust domain estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTrustDomainEstimate {
        TieredTrustDomainEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            batch_size: self.batch_size,
            visible: self.visible,
            priority: self.priority,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered trust domain estimate in status idle.
pub fn new_tiered_trust_domain_estimate(code: &str, label: &str) -> TieredTrustDomainEstimate {
    TieredTrustDomainEstimate {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        batch_size: 0,
        visible: false,
        priority: 0,
        cluster: "".to_string(),
        status: TieredTrustDomainEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_trust_domain_estimate("code-439", "label-546");
        assert_eq!(item.status, TieredTrustDomainEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_trust_domain_estimate("code-439", "label-546");
        if item.move_to(TieredTrustDomainEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTrustDomainEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_trust_domain_estimate("code-439", "label-546");
        item.status = TieredTrustDomainEstimateStatus::Stopped;
        assert!(
            item.move_to(TieredTrustDomainEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
