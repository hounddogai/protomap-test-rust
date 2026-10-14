use super::*;

use acme_kit::text;

/// Where a failover quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FailoverQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FailoverQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover quota as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuota {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub reviewer: String,
    pub threshold: f64,
    pub limit: i64,
    pub tenant_id: String,
    pub status: FailoverQuotaStatus,
}

impl FailoverQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverQuotaStatus) -> bool {
        match self.status {
            FailoverQuotaStatus::Created => {
                return next == FailoverQuotaStatus::Published;
            }
            FailoverQuotaStatus::Validated => {
                match next {
                    FailoverQuotaStatus::Published | FailoverQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverQuotaStatus::Published => {
                return next == FailoverQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "failover quota", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.limit as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverQuota {
        FailoverQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            reviewer: self.reviewer.clone(),
            threshold: self.threshold,
            limit: self.limit,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a failover quota in status created.
pub fn new_failover_quota(code: &str, name: &str) -> FailoverQuota {
    FailoverQuota {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        reviewer: "".to_string(),
        threshold: 0.0,
        limit: 0,
        tenant_id: "".to_string(),
        status: FailoverQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_failover_quota("code-584", "name-14");
        assert_eq!(item.status, FailoverQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_failover_quota("code-584", "name-14");
        if item.move_to(FailoverQuotaStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverQuotaStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_failover_quota("code-584", "name-14");
        item.status = FailoverQuotaStatus::Retired;
        assert!(
            item.move_to(FailoverQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
