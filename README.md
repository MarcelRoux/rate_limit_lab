# rate_limit_lab
The Rate Limiter Evaluation Framework is a protocol-agnostic system for designing, testing, and empirically evaluating rate-limiting strategies.

## Development Environment (M0)

Before starting development, ensure the repository hooks and tooling are installed.

### 1. Install Git Hooks
Run the installation script:

```bash
. ./scripts/install-hooks.sh
```

## Traffic Generator Design Notes

The REST traffic generator used throughout this project was developed with an emphasis on **repeatability, interpretability, and controlled tradeoffs**, rather than raw maximum throughput alone.

During M2.4, multiple pacing and execution models were evaluated (including queue-based worker pools) to understand their impact on load fidelity, concurrency behavior, and system observability. The final design intentionally favors a simpler batch-per-tick model that is sufficient for single-node and early distributed experiments, while remaining extensible.

A detailed discussion of the rationale, alternatives considered, empirical findings, and future improvement areas is documented here:

📄 **[Traffic Generator Design Notes](docs/design/traffic_generator.md)**

This document serves as architectural context for current and future milestones (M3+), and explains why certain complexity was deferred in favor of clarity and correctness at this stage.

## License

This project is licensed under the Apache License 2.0.
See the LICENSE file for details.

## AI Usage Notice

This repository is licensed under the Apache License 2.0.

The author does not grant permission for this codebase, documentation,
or derived artifacts to be used for training machine learning or
artificial intelligence models, except where such use is explicitly
permitted under the terms of the Apache License 2.0.

Automated scraping for the purpose of dataset aggregation or model
training is discouraged.

## Automated Access Policy

This repository includes a robots.txt file expressing the author's
intent to disallow automated scraping for AI training and dataset
aggregation purposes.

While robots.txt is advisory, it serves as an explicit declaration
of usage intent.

## Tooling

Enforces conventional commits, static checks, spelling validation, and test gating via custom Git hooks.