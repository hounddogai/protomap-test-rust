use super::*;

use acme_kit::text;

/// Where a tiered silence policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSilencePolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredSilencePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TieredSilencePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered silence policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicy {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub tenant_id: String,
    pub created_at: i64,
    pub discount: f64,
    pub weight: f64,
    pub status: TieredSilencePolicyStatus,
}

impl TieredSilencePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSilencePolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSilencePolicyStatus) -> bool {
        match self.status {
            TieredSilencePolicyStatus::Planned => {
                match next {
                    TieredSilencePolicyStatus::Completed | TieredSilencePolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSilencePolicyStatus::Scheduled => {
                match next {
                    TieredSilencePolicyStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSilencePolicyStatus::InProgress => {
                match next {
                    TieredSilencePolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSilencePolicyStatus::Completed => {
                match next {
                    TieredSilencePolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredSilencePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered silence policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered silence policy", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.discount * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSilencePolicy {
        TieredSilencePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            discount: self.discount,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered silence policy in status planned.
pub fn new_tiered_silence_policy(code: &str, name: &str) -> TieredSilencePolicy {
    TieredSilencePolicy {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        tenant_id: "".to_string(),
        created_at: 0,
        discount: 0.0,
        weight: 0.0,
        status: TieredSilencePolicyStatus::Planned,
    }
}
