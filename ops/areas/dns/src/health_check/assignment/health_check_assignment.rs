use super::*;

use acme_kit::text;

/// Where a health check assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl HealthCheckAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for HealthCheckAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAssignment {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub visible: bool,
    pub title: String,
    pub limit: i64,
    pub retries: i64,
    pub ceiling: f64,
    pub cluster: String,
    pub status: HealthCheckAssignmentStatus,
}

impl HealthCheckAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckAssignmentStatus) -> bool {
        match self.status {
            HealthCheckAssignmentStatus::Created => {
                match next {
                    HealthCheckAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAssignmentStatus::Validated => {
                match next {
                    HealthCheckAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAssignmentStatus::Published => {
                match next {
                    HealthCheckAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: HealthCheckAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        (self.ceiling * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckAssignment {
        HealthCheckAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            visible: self.visible,
            title: self.title.clone(),
            limit: self.limit,
            retries: self.retries,
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a health check assignment in status created.
pub fn new_health_check_assignment(code: &str, label: &str) -> HealthCheckAssignment {
    HealthCheckAssignment {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        visible: false,
        title: "".to_string(),
        limit: 0,
        retries: 0,
        ceiling: 0.0,
        cluster: "".to_string(),
        status: HealthCheckAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_health_check_assignment("code-229", "label-274");
        assert_eq!(item.status, HealthCheckAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_health_check_assignment("code-229", "label-274");
        if item.move_to(HealthCheckAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_health_check_assignment("code-229", "label-274");
        item.status = HealthCheckAssignmentStatus::Retired;
        assert!(
            item.move_to(HealthCheckAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
