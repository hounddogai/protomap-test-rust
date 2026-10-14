use super::*;

use acme_kit::text;

/// Where a limit assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LimitAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LimitAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignment {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub checksum: String,
    pub cluster: String,
    pub margin: f64,
    pub unit: String,
    pub ttl_seconds: i64,
    pub reviewer: String,
    pub namespace: String,
    pub title: String,
    pub status: LimitAssignmentStatus,
}

impl LimitAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitAssignmentStatus) -> bool {
        match self.status {
            LimitAssignmentStatus::Created => {
                return next == LimitAssignmentStatus::Published || next == LimitAssignmentStatus::Retired;
            }
            LimitAssignmentStatus::Validated => {
                match next {
                    LimitAssignmentStatus::Published | LimitAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAssignmentStatus::Published => {
                return next == LimitAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "limit assignment", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitAssignment {
        LimitAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            margin: self.margin,
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            reviewer: self.reviewer.clone(),
            namespace: self.namespace.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a limit assignment in status created.
pub fn new_limit_assignment(code: &str, label: &str) -> LimitAssignment {
    LimitAssignment {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        checksum: "".to_string(),
        cluster: "".to_string(),
        margin: 0.0,
        unit: "".to_string(),
        ttl_seconds: 0,
        reviewer: "".to_string(),
        namespace: "".to_string(),
        title: "".to_string(),
        status: LimitAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_limit_assignment("code-227", "label-608");
        assert_eq!(item.status, LimitAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_limit_assignment("code-227", "label-608");
        if item.move_to(LimitAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_limit_assignment("code-227", "label-608");
        item.status = LimitAssignmentStatus::Retired;
        assert!(
            item.move_to(LimitAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
