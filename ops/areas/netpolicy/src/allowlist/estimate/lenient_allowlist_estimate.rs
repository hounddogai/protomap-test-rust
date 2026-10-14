use super::*;

use acme_kit::text;

/// Where a lenient allowlist estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAllowlistEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientAllowlistEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientAllowlistEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient allowlist estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimate {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub issuer: String,
    pub amount: f64,
    pub ceiling: f64,
    pub fingerprint: String,
    pub rotation_days: i64,
    pub port: i64,
    pub unit: String,
    pub archived: bool,
    pub status: LenientAllowlistEstimateStatus,
}

impl LenientAllowlistEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAllowlistEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAllowlistEstimateStatus) -> bool {
        match self.status {
            LenientAllowlistEstimateStatus::Created => {
                match next {
                    LenientAllowlistEstimateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistEstimateStatus::Validated => {
                match next {
                    LenientAllowlistEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistEstimateStatus::Published => {
                return next == LenientAllowlistEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientAllowlistEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient allowlist estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.ceiling * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAllowlistEstimate {
        LenientAllowlistEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            issuer: self.issuer.clone(),
            amount: self.amount,
            ceiling: self.ceiling,
            fingerprint: self.fingerprint.clone(),
            rotation_days: self.rotation_days,
            port: self.port,
            unit: self.unit.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient allowlist estimate in status created.
pub fn new_lenient_allowlist_estimate(code: &str, label: &str) -> LenientAllowlistEstimate {
    LenientAllowlistEstimate {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        issuer: "".to_string(),
        amount: 0.0,
        ceiling: 0.0,
        fingerprint: "".to_string(),
        rotation_days: 0,
        port: 0,
        unit: "".to_string(),
        archived: false,
        status: LenientAllowlistEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_allowlist_estimate("code-802", "label-158");
        assert_eq!(item.status, LenientAllowlistEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_allowlist_estimate("code-802", "label-158");
        if item.move_to(LenientAllowlistEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAllowlistEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_allowlist_estimate("code-802", "label-158");
        item.status = LenientAllowlistEstimateStatus::Retired;
        assert!(
            item.move_to(LenientAllowlistEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
