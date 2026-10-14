use super::*;

use acme_kit::text;

/// Where a reservation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAudit {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub cost: f64,
    pub tags: Vec<String>,
    pub burst: i64,
    pub name: String,
    pub rate: f64,
    pub source: String,
    pub ttl_seconds: i64,
    pub status: ReservationAuditStatus,
}

impl ReservationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationAuditStatus) -> bool {
        match self.status {
            ReservationAuditStatus::Proposed => {
                return next == ReservationAuditStatus::Approved;
            }
            ReservationAuditStatus::Approved => {
                return next == ReservationAuditStatus::Rejected || next == ReservationAuditStatus::Withdrawn;
            }
            ReservationAuditStatus::Rejected => {
                match next {
                    ReservationAuditStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReservationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.cost + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationAudit {
        ReservationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            tags: self.tags.clone(),
            burst: self.burst,
            name: self.name.clone(),
            rate: self.rate,
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a reservation audit in status proposed.
pub fn new_reservation_audit(code: &str, title: &str) -> ReservationAudit {
    ReservationAudit {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        tags: Vec::new(),
        burst: 0,
        name: "".to_string(),
        rate: 0.0,
        source: "".to_string(),
        ttl_seconds: 0,
        status: ReservationAuditStatus::Proposed,
    }
}
