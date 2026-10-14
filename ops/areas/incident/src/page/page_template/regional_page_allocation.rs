use super::*;

use acme_kit::text;

/// Where a regional page allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPageAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalPageAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalPageAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional page allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageAllocation {
    pub code: String,
    pub name: String,
    pub description: String,
    pub threshold: f64,
    pub notes: String,
    pub burst: i64,
    pub status: RegionalPageAllocationStatus,
}

impl RegionalPageAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPageAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPageAllocationStatus) -> bool {
        match self.status {
            RegionalPageAllocationStatus::Created => {
                match next {
                    RegionalPageAllocationStatus::Validated | RegionalPageAllocationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPageAllocationStatus::Validated => {
                return next == RegionalPageAllocationStatus::Published
                    || next == RegionalPageAllocationStatus::Retired;
            }
            RegionalPageAllocationStatus::Published => {
                return next == RegionalPageAllocationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPageAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional page allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.threshold * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPageAllocation {
        RegionalPageAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            threshold: self.threshold,
            notes: self.notes.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional page allocation in status created.
pub fn new_regional_page_allocation(code: &str, name: &str) -> RegionalPageAllocation {
    RegionalPageAllocation {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        threshold: 0.0,
        notes: "".to_string(),
        burst: 0,
        status: RegionalPageAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_page_allocation("code-409", "name-470");
        assert_eq!(item.status, RegionalPageAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_page_allocation("code-409", "name-470");
        if item.move_to(RegionalPageAllocationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPageAllocationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_page_allocation("code-409", "name-470");
        item.status = RegionalPageAllocationStatus::Retired;
        assert!(
            item.move_to(RegionalPageAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
