use super::*;

/// How screens and exports show a peering policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicyView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub ttl_seconds_text: String,
    pub ceiling: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_peering_policy_view(item: &PeeringPolicy) -> PeeringPolicyView {
    PeeringPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_policy_views(items: &[PeeringPolicy]) -> Vec<PeeringPolicyView> {
    let mut views: Vec<PeeringPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_policy_view(item));
    }
    views
}

pub fn sum_peering_policy_view(view: &PeeringPolicyView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_policy("code-105", "name-817");
        let view = to_peering_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
