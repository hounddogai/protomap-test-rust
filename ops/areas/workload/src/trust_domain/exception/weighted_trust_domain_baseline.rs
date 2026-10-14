use super::*;

use acme_kit::text;

/// Where a weighted trust domain baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTrustDomainBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedTrustDomainBaselineStatus {
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

impl std::fmt::Display for WeightedTrustDomainBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted trust domain baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainBaseline {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub cost: f64,
    pub quota_units: i64,
    pub namespace: String,
    pub approved: bool,
    pub budget: f64,
    pub status: WeightedTrustDomainBaselineStatus,
}

impl WeightedTrustDomainBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTrustDomainBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTrustDomainBaselineStatus) -> bool {
        match self.status {
            WeightedTrustDomainBaselineStatus::Idle => {
                match next {
                    WeightedTrustDomainBaselineStatus::Warming | WeightedTrustDomainBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainBaselineStatus::Warming => {
                match next {
                    WeightedTrustDomainBaselineStatus::Draining | WeightedTrustDomainBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainBaselineStatus::Ready => {
                match next {
                    WeightedTrustDomainBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainBaselineStatus::Draining => {
                match next {
                    WeightedTrustDomainBaselineStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedTrustDomainBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted trust domain baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTrustDomainBaseline {
        WeightedTrustDomainBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            cost: self.cost,
            quota_units: self.quota_units,
            namespace: self.namespace.clone(),
            approved: self.approved,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a weighted trust domain baseline in status idle.
pub fn new_weighted_trust_domain_baseline(code: &str, name: &str) -> WeightedTrustDomainBaseline {
    WeightedTrustDomainBaseline {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        cost: 0.0,
        quota_units: 0,
        namespace: "".to_string(),
        approved: false,
        budget: 0.0,
        status: WeightedTrustDomainBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_trust_domain_baseline("code-12", "name-138");
        assert_eq!(item.status, WeightedTrustDomainBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_trust_domain_baseline("code-12", "name-138");
        if item.move_to(WeightedTrustDomainBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTrustDomainBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_trust_domain_baseline("code-12", "name-138");
        item.status = WeightedTrustDomainBaselineStatus::Stopped;
        assert!(
            item.move_to(WeightedTrustDomainBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
