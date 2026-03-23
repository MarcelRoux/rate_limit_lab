pub(crate) fn select_ats(profile: Option<String>, at: Option<String>) -> Vec<String> {
    match (profile, at) {
        (Some(p), None) if p == "smoke_ready" => vec![
            "AT-004", "AT-005", "AT-006", "AT-007", "AT-008", "AT-009", "AT-010", "AT-011",
            "AT-012", "AT-013", "AT-014", "AT-015", "AT-018", "AT-019", "AT-020", "AT-021",
            "AT-022", "AT-023", "AT-024", "AT-050",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        (Some(p), None) if p == "full_matrix" => vec![
            "AT-004", "AT-005", "AT-006", "AT-007", "AT-008", "AT-009", "AT-010", "AT-011",
            "AT-012", "AT-013", "AT-014", "AT-015", "AT-016", "AT-017", "AT-018", "AT-019",
            "AT-020", "AT-021", "AT-022", "AT-023", "AT-024", "AT-025", "AT-026", "AT-027",
            "AT-028", "AT-029", "AT-030", "AT-031", "AT-032", "AT-033", "AT-034", "AT-050",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        (Some(p), None) if p == "rr_profile" => vec!["AT-030", "AT-032", "AT-034"]
            .into_iter()
            .map(String::from)
            .collect(),
        (Some(p), None) if p == "sa_profile" => vec!["AT-031", "AT-033", "AT-034"]
            .into_iter()
            .map(String::from)
            .collect(),
        (Some(p), None) if p == "failure_matrix" => {
            vec!["AT-025", "AT-026", "AT-027", "AT-028", "AT-029"]
                .into_iter()
                .map(String::from)
                .collect()
        }
        (Some(p), None) if p == "observability_mvp" => {
            vec!["AT-052", "AT-053", "AT-054", "AT-055", "AT-056"]
                .into_iter()
                .map(String::from)
                .collect()
        }
        (Some(_), None) => vec![String::from("AT-004")],
        (None, Some(single)) => vec![single],
        _ => Vec::new(),
    }
}
