use super::*;

use acme_kit::text;

/// Where a scheduled trust domain budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTrustDomainBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledTrustDomainBudgetStatus {
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

impl std::fmt::Display for ScheduledTrustDomainBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled trust domain budget as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainBudget {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub contact_email: String,
    pub cluster: String,
    pub country: String,
    pub name: String,
    pub maximum: f64,
    pub threshold: f64,
    pub ends_at: i64,
    pub starts_at: i64,
    pub status: ScheduledTrustDomainBudgetStatus,
}

impl ScheduledTrustDomainBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTrustDomainBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTrustDomainBudgetStatus) -> bool {
        match self.status {
            ScheduledTrustDomainBudgetStatus::Planned => {
                match next {
                    ScheduledTrustDomainBudgetStatus::InProgress | ScheduledTrustDomainBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTrustDomainBudgetStatus::Scheduled => {
                match next {
                    ScheduledTrustDomainBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTrustDomainBudgetStatus::InProgress => {
                match next {
                    ScheduledTrustDomainBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTrustDomainBudgetStatus::Completed => {
                return next == ScheduledTrustDomainBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTrustDomainBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled trust domain budget {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.maximum + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTrustDomainBudget {
        ScheduledTrustDomainBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            contact_email: self.contact_email.clone(),
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            threshold: self.threshold,
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled trust domain budget in status planned.
pub fn new_scheduled_trust_domain_budget(code: &str, label: &str) -> ScheduledTrustDomainBudget {
    ScheduledTrustDomainBudget {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        contact_email: "".to_string(),
        cluster: "".to_string(),
        country: "".to_string(),
        name: "".to_string(),
        maximum: 0.0,
        threshold: 0.0,
        ends_at: 0,
        starts_at: 0,
        status: ScheduledTrustDomainBudgetStatus::Planned,
    }
}
