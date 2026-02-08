fn main() {
    // List the mutually exclusive “variant” features here.
    const VARIANTS: &[&str] = &[
        "in_memory_limiter",
        "distributed_limiter",
        "hybrid_limiter",
        // add more here
    ];

    let enabled: Vec<&str> = VARIANTS
        .iter()
        .copied()
        .filter(|f| std::env::var(format!("CARGO_FEATURE_{}", f.to_uppercase())).is_ok())
        .collect();

    if enabled.len() != 1 {
        panic!(
            "Exactly one REST limiter variant feature must be enabled.\nEnabled: {:?}.\nValid variants: {:?}.",
            enabled, VARIANTS
        );
    }
}
