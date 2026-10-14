use super::*;

use acme_kit::text;

/// Where a policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PolicyVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy variance as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVariance {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub ends_at: i64,
    pub starts_at: i64,
    pub tenant_id: String,
    pub status: PolicyVarianceStatus,
}

impl PolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyVarianceStatus) -> bool {
        match self.status {
            PolicyVarianceStatus::Created => {
                match next {
                    PolicyVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyVarianceStatus::Validated => {
                match next {
                    PolicyVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyVarianceStatus::Published => {
                return next == PolicyVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy variance", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.margin * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyVariance {
        PolicyVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a policy variance in status created.
pub fn new_policy_variance(code: &str, title: &str) -> PolicyVariance {
    PolicyVariance {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        ends_at: 0,
        starts_at: 0,
        tenant_id: "".to_string(),
        status: PolicyVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_policy_variance("code-753", "title-34");
        assert_eq!(item.status, PolicyVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_policy_variance("code-753", "title-34");
        if item.move_to(PolicyVarianceStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyVarianceStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_policy_variance("code-753", "title-34");
        item.status = PolicyVarianceStatus::Retired;
        assert!(
            item.move_to(PolicyVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
