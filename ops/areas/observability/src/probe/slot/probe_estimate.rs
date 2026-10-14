use super::*;

use acme_kit::text;

/// Where a probe estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimate {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub channel: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub checksum: String,
    pub tenant_id: String,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub status: ProbeEstimateStatus,
}

impl ProbeEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeEstimateStatus) -> bool {
        match self.status {
            ProbeEstimateStatus::Created => {
                return next == ProbeEstimateStatus::Retired;
            }
            ProbeEstimateStatus::Validated => {
                return next == ProbeEstimateStatus::Published || next == ProbeEstimateStatus::Retired;
            }
            ProbeEstimateStatus::Published => {
                return next == ProbeEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeEstimate {
        ProbeEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            channel: self.channel.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a probe estimate in status created.
pub fn new_probe_estimate(code: &str, name: &str) -> ProbeEstimate {
    ProbeEstimate {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        channel: "".to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        attempts: 0,
        ttl_seconds: 0,
        status: ProbeEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_estimate("code-820", "name-270");
        assert_eq!(item.status, ProbeEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_probe_estimate("code-820", "name-270");
        if item.move_to(ProbeEstimateStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeEstimateStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_estimate("code-820", "name-270");
        item.status = ProbeEstimateStatus::Retired;
        assert!(
            item.move_to(ProbeEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
