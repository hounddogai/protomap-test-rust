use super::*;

use acme_kit::text;

/// Where a lenient evidence plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEvidencePlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientEvidencePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientEvidencePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient evidence plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidencePlan {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub margin: f64,
    pub sequence: i64,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub scope: String,
    pub ceiling: f64,
    pub channel: String,
    pub port: i64,
    pub status: LenientEvidencePlanStatus,
}

impl LenientEvidencePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEvidencePlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEvidencePlanStatus) -> bool {
        match self.status {
            LenientEvidencePlanStatus::Created => {
                return next == LenientEvidencePlanStatus::Validated;
            }
            LenientEvidencePlanStatus::Validated => {
                match next {
                    LenientEvidencePlanStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEvidencePlanStatus::Published => {
                match next {
                    LenientEvidencePlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientEvidencePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient evidence plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient evidence plan", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.sequence as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEvidencePlan {
        LenientEvidencePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            margin: self.margin,
            sequence: self.sequence,
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            channel: self.channel.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient evidence plan in status created.
pub fn new_lenient_evidence_plan(code: &str, label: &str) -> LenientEvidencePlan {
    LenientEvidencePlan {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        margin: 0.0,
        sequence: 0,
        pinned: false,
        ttl_seconds: 0,
        scope: "".to_string(),
        ceiling: 0.0,
        channel: "".to_string(),
        port: 0,
        status: LenientEvidencePlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_evidence_plan("code-9", "label-441");
        assert_eq!(item.status, LenientEvidencePlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_evidence_plan("code-9", "label-441");
        if item.move_to(LenientEvidencePlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEvidencePlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_evidence_plan("code-9", "label-441");
        item.status = LenientEvidencePlanStatus::Retired;
        assert!(
            item.move_to(LenientEvidencePlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
