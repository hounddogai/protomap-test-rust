use super::*;

use acme_kit::text;

/// Where a audit event metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AuditEventMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AuditEventMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event metric as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetric {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cluster: String,
    pub fingerprint: String,
    pub limit: i64,
    pub margin: f64,
    pub description: String,
    pub quantity: i64,
    pub weight: f64,
    pub status: AuditEventMetricStatus,
}

impl AuditEventMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventMetricStatus) -> bool {
        match self.status {
            AuditEventMetricStatus::Draft => {
                return next == AuditEventMetricStatus::Active || next == AuditEventMetricStatus::Closed;
            }
            AuditEventMetricStatus::Pending => {
                return next == AuditEventMetricStatus::Active || next == AuditEventMetricStatus::Closed;
            }
            AuditEventMetricStatus::Active => {
                return next == AuditEventMetricStatus::Closed;
            }
            AuditEventMetricStatus::Suspended => {
                match next {
                    AuditEventMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: AuditEventMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event metric", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventMetric {
        AuditEventMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            cluster: self.cluster.clone(),
            fingerprint: self.fingerprint.clone(),
            limit: self.limit,
            margin: self.margin,
            description: self.description.clone(),
            quantity: self.quantity,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a audit event metric in status draft.
pub fn new_audit_event_metric(code: &str, name: &str) -> AuditEventMetric {
    AuditEventMetric {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        cluster: "".to_string(),
        fingerprint: "".to_string(),
        limit: 0,
        margin: 0.0,
        description: "".to_string(),
        quantity: 0,
        weight: 0.0,
        status: AuditEventMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_audit_event_metric("code-919", "name-401");
        assert_eq!(item.status, AuditEventMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_audit_event_metric("code-919", "name-401");
        if item.move_to(AuditEventMetricStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventMetricStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_audit_event_metric("code-919", "name-401");
        item.status = AuditEventMetricStatus::Closed;
        assert!(
            item.move_to(AuditEventMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
