use super::*;

use acme_kit::text;

/// Where a adaptive tenant preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTenantPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveTenantPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveTenantPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive tenant preference as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantPreference {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub fingerprint: String,
    pub rate: f64,
    pub score: f64,
    pub status: AdaptiveTenantPreferenceStatus,
}

impl AdaptiveTenantPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTenantPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTenantPreferenceStatus) -> bool {
        match self.status {
            AdaptiveTenantPreferenceStatus::New => {
                return next == AdaptiveTenantPreferenceStatus::Done;
            }
            AdaptiveTenantPreferenceStatus::Queued => {
                return next == AdaptiveTenantPreferenceStatus::Running || next == AdaptiveTenantPreferenceStatus::Done;
            }
            AdaptiveTenantPreferenceStatus::Running => {
                match next {
                    AdaptiveTenantPreferenceStatus::Done | AdaptiveTenantPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveTenantPreferenceStatus::Done => {
                return next == AdaptiveTenantPreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveTenantPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive tenant preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.maximum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTenantPreference {
        AdaptiveTenantPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            rate: self.rate,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive tenant preference in status new.
pub fn new_adaptive_tenant_preference(code: &str, name: &str) -> AdaptiveTenantPreference {
    AdaptiveTenantPreference {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        fingerprint: "".to_string(),
        rate: 0.0,
        score: 0.0,
        status: AdaptiveTenantPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_tenant_preference("code-279", "name-724");
        assert_eq!(item.status, AdaptiveTenantPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_adaptive_tenant_preference("code-279", "name-724");
        if item.move_to(AdaptiveTenantPreferenceStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveTenantPreferenceStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_tenant_preference("code-279", "name-724");
        item.status = AdaptiveTenantPreferenceStatus::Failed;
        assert!(
            item.move_to(AdaptiveTenantPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
