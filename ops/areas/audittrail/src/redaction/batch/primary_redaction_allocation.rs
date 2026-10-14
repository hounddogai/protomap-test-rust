use super::*;

use acme_kit::text;

/// Where a primary redaction allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRedactionAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryRedactionAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryRedactionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary redaction allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionAllocation {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub quantity: i64,
    pub description: String,
    pub grace_days: i64,
    pub starts_at: i64,
    pub port: i64,
    pub window_days: i64,
    pub unit: String,
    pub minimum: f64,
    pub status: PrimaryRedactionAllocationStatus,
}

impl PrimaryRedactionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRedactionAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRedactionAllocationStatus) -> bool {
        match self.status {
            PrimaryRedactionAllocationStatus::Created => {
                match next {
                    PrimaryRedactionAllocationStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRedactionAllocationStatus::Validated => {
                return next == PrimaryRedactionAllocationStatus::Published
                    || next == PrimaryRedactionAllocationStatus::Retired;
            }
            PrimaryRedactionAllocationStatus::Published => {
                match next {
                    PrimaryRedactionAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryRedactionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary redaction allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.window_days as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRedactionAllocation {
        PrimaryRedactionAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            description: self.description.clone(),
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            port: self.port,
            window_days: self.window_days,
            unit: self.unit.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary redaction allocation in status created.
pub fn new_primary_redaction_allocation(code: &str, title: &str) -> PrimaryRedactionAllocation {
    PrimaryRedactionAllocation {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        quantity: 0,
        description: "".to_string(),
        grace_days: 0,
        starts_at: 0,
        port: 0,
        window_days: 0,
        unit: "".to_string(),
        minimum: 0.0,
        status: PrimaryRedactionAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_redaction_allocation("code-830", "title-723");
        assert_eq!(item.status, PrimaryRedactionAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_redaction_allocation("code-830", "title-723");
        if item.move_to(PrimaryRedactionAllocationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRedactionAllocationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_redaction_allocation("code-830", "title-723");
        item.status = PrimaryRedactionAllocationStatus::Retired;
        assert!(
            item.move_to(PrimaryRedactionAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
