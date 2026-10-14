use super::*;

use acme_kit::text;

/// Where a dns record adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordAdjustmentStatus {
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

impl std::fmt::Display for DnsRecordAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustment {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub unit: String,
    pub score: f64,
    pub quota_units: i64,
    pub issuer: String,
    pub margin: f64,
    pub revision: i64,
    pub cost: f64,
    pub status: DnsRecordAdjustmentStatus,
}

impl DnsRecordAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordAdjustmentStatus) -> bool {
        match self.status {
            DnsRecordAdjustmentStatus::Idle => {
                match next {
                    DnsRecordAdjustmentStatus::Warming | DnsRecordAdjustmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAdjustmentStatus::Warming => {
                return next == DnsRecordAdjustmentStatus::Stopped;
            }
            DnsRecordAdjustmentStatus::Ready => {
                match next {
                    DnsRecordAdjustmentStatus::Draining | DnsRecordAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAdjustmentStatus::Draining => {
                return next == DnsRecordAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.cost - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordAdjustment {
        DnsRecordAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            score: self.score,
            quota_units: self.quota_units,
            issuer: self.issuer.clone(),
            margin: self.margin,
            revision: self.revision,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a dns record adjustment in status idle.
pub fn new_dns_record_adjustment(code: &str, name: &str) -> DnsRecordAdjustment {
    DnsRecordAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        score: 0.0,
        quota_units: 0,
        issuer: "".to_string(),
        margin: 0.0,
        revision: 0,
        cost: 0.0,
        status: DnsRecordAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_adjustment("code-588", "name-749");
        assert_eq!(item.status, DnsRecordAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dns_record_adjustment("code-588", "name-749");
        if item.move_to(DnsRecordAdjustmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordAdjustmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_adjustment("code-588", "name-749");
        item.status = DnsRecordAdjustmentStatus::Stopped;
        assert!(
            item.move_to(DnsRecordAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
