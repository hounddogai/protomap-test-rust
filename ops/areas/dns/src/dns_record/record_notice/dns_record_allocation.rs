use super::*;

use acme_kit::text;

/// Where a dns record allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DnsRecordAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DnsRecordAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record allocation as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAllocation {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub weight: f64,
    pub principal: String,
    pub amount: f64,
    pub version: i64,
    pub maximum: f64,
    pub cluster: String,
    pub cost: f64,
    pub source: String,
    pub status: DnsRecordAllocationStatus,
}

impl DnsRecordAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordAllocationStatus) -> bool {
        match self.status {
            DnsRecordAllocationStatus::Created => {
                match next {
                    DnsRecordAllocationStatus::Validated | DnsRecordAllocationStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAllocationStatus::Validated => {
                return next == DnsRecordAllocationStatus::Published || next == DnsRecordAllocationStatus::Retired;
            }
            DnsRecordAllocationStatus::Published => {
                match next {
                    DnsRecordAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: DnsRecordAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record allocation", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.amount + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordAllocation {
        DnsRecordAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            weight: self.weight,
            principal: self.principal.clone(),
            amount: self.amount,
            version: self.version,
            maximum: self.maximum,
            cluster: self.cluster.clone(),
            cost: self.cost,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a dns record allocation in status created.
pub fn new_dns_record_allocation(code: &str, name: &str) -> DnsRecordAllocation {
    DnsRecordAllocation {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        weight: 0.0,
        principal: "".to_string(),
        amount: 0.0,
        version: 0,
        maximum: 0.0,
        cluster: "".to_string(),
        cost: 0.0,
        source: "".to_string(),
        status: DnsRecordAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_dns_record_allocation("code-406", "name-178");
        assert_eq!(item.status, DnsRecordAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_dns_record_allocation("code-406", "name-178");
        if item.move_to(DnsRecordAllocationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordAllocationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_dns_record_allocation("code-406", "name-178");
        item.status = DnsRecordAllocationStatus::Retired;
        assert!(
            item.move_to(DnsRecordAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
