use super::*;

use acme_kit::text;

/// How screens and exports show a failover variance.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub phone: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_failover_variance_view(item: &FailoverVariance) -> FailoverVarianceView {
    FailoverVarianceView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        phone: text::truncate(&item.phone, 120),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_variance_views(items: &[FailoverVariance]) -> Vec<FailoverVarianceView> {
    let mut views: Vec<FailoverVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_variance_view(item));
    }
    views
}

pub fn sum_failover_variance_view(view: &FailoverVarianceView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_variance("code-830", "name-909");
        let view = to_failover_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
