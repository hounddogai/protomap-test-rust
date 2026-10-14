use super::*;

use acme_kit::text;

/// Where a strict suspension rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSuspensionRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictSuspensionRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictSuspensionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict suspension rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionRollup {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub principal: String,
    pub batch_size: i64,
    pub status: StrictSuspensionRollupStatus,
}

impl StrictSuspensionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSuspensionRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSuspensionRollupStatus) -> bool {
        match self.status {
            StrictSuspensionRollupStatus::Created => {
                match next {
                    StrictSuspensionRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSuspensionRollupStatus::Validated => {
                match next {
                    StrictSuspensionRollupStatus::Published | StrictSuspensionRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSuspensionRollupStatus::Published => {
                match next {
                    StrictSuspensionRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictSuspensionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict suspension rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> StrictSuspensionRollup {
        StrictSuspensionRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict suspension rollup in status created.
pub fn new_strict_suspension_rollup(code: &str, title: &str) -> StrictSuspensionRollup {
    StrictSuspensionRollup {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        principal: "".to_string(),
        batch_size: 0,
        status: StrictSuspensionRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_suspension_rollup("code-388", "title-471");
        assert_eq!(item.status, StrictSuspensionRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_strict_suspension_rollup("code-388", "title-471");
        if item.move_to(StrictSuspensionRollupStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSuspensionRollupStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_suspension_rollup("code-388", "title-471");
        item.status = StrictSuspensionRollupStatus::Retired;
        assert!(
            item.move_to(StrictSuspensionRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
