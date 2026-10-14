use super::*;

use acme_kit::text;

/// Where a fallback dns record allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDnsRecordAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackDnsRecordAllocationStatus {
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

impl std::fmt::Display for FallbackDnsRecordAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback dns record allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocation {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub minimum: f64,
    pub margin: f64,
    pub weight: f64,
    pub maximum: f64,
    pub ceiling: f64,
    pub approved: bool,
    pub updated_at: i64,
    pub audience: String,
    pub status: FallbackDnsRecordAllocationStatus,
}

impl FallbackDnsRecordAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDnsRecordAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDnsRecordAllocationStatus) -> bool {
        match self.status {
            FallbackDnsRecordAllocationStatus::Draft => {
                match next {
                    FallbackDnsRecordAllocationStatus::Active | FallbackDnsRecordAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDnsRecordAllocationStatus::Pending => {
                match next {
                    FallbackDnsRecordAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDnsRecordAllocationStatus::Active => {
                return next == FallbackDnsRecordAllocationStatus::Suspended;
            }
            FallbackDnsRecordAllocationStatus::Suspended => {
                match next {
                    FallbackDnsRecordAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackDnsRecordAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback dns record allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ceiling - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDnsRecordAllocation {
        FallbackDnsRecordAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            minimum: self.minimum,
            margin: self.margin,
            weight: self.weight,
            maximum: self.maximum,
            ceiling: self.ceiling,
            approved: self.approved,
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback dns record allocation in status draft.
pub fn new_fallback_dns_record_allocation(code: &str, name: &str) -> FallbackDnsRecordAllocation {
    FallbackDnsRecordAllocation {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        minimum: 0.0,
        margin: 0.0,
        weight: 0.0,
        maximum: 0.0,
        ceiling: 0.0,
        approved: false,
        updated_at: 0,
        audience: "".to_string(),
        status: FallbackDnsRecordAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_dns_record_allocation("code-350", "name-784");
        assert_eq!(item.status, FallbackDnsRecordAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_dns_record_allocation("code-350", "name-784");
        if item.move_to(FallbackDnsRecordAllocationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDnsRecordAllocationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_dns_record_allocation("code-350", "name-784");
        item.status = FallbackDnsRecordAllocationStatus::Closed;
        assert!(
            item.move_to(FallbackDnsRecordAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
