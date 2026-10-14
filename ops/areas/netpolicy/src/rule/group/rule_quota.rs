use super::*;

use acme_kit::text;

/// Where a rule quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RuleQuotaStatus {
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

impl std::fmt::Display for RuleQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuota {
    pub code: String,
    pub name: String,
    pub description: String,
    pub country: String,
    pub locale: String,
    pub phone: String,
    pub namespace: String,
    pub cost: f64,
    pub capacity: i64,
    pub rate: f64,
    pub updated_at: i64,
    pub status: RuleQuotaStatus,
}

impl RuleQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleQuotaStatus) -> bool {
        match self.status {
            RuleQuotaStatus::New => {
                match next {
                    RuleQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleQuotaStatus::Queued => {
                match next {
                    RuleQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleQuotaStatus::Running => {
                return next == RuleQuotaStatus::Done || next == RuleQuotaStatus::Failed;
            }
            RuleQuotaStatus::Done => {
                match next {
                    RuleQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: RuleQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule quota", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rate + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleQuota {
        RuleQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            country: self.country.clone(),
            locale: self.locale.clone(),
            phone: self.phone.clone(),
            namespace: self.namespace.clone(),
            cost: self.cost,
            capacity: self.capacity,
            rate: self.rate,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rule quota in status new.
pub fn new_rule_quota(code: &str, name: &str) -> RuleQuota {
    RuleQuota {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        country: "".to_string(),
        locale: "".to_string(),
        phone: "".to_string(),
        namespace: "".to_string(),
        cost: 0.0,
        capacity: 0,
        rate: 0.0,
        updated_at: 0,
        status: RuleQuotaStatus::New,
    }
}
