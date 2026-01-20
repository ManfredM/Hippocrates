# Hippocrates

**Hippocrates** is a domain-specific language (DSL) for defining medical care plans, protocols, and digital health interventions. It emphasizes readability, type safety, and rigorous validation of medical data.

## Key Features

- **Natural Language Syntax**: Designed to be readable by medical professionals.
- **Strict Unit Validation**: All numeric values must have units (e.g., `10 mg`, `5 steps`). Users explicitly define units and their plural forms (e.g., `drop` vs `drops`) to ensure precise handling.
- **Data Precision**: Explicit handling of integer vs. float precision (e.g., `0 ... 10` vs `0.0 ... 10.0`).
- **Double-Entry Validation**: Syntax support for ensuring data accuracy:

  ```hippocrates
  ask "Enter dosage":
      validate answer twice.
  ```

- **Event-Driven**: Reacts to time, value changes, and external triggers.

## Documentation

- [Language Specification](specification/hippocrates_specification.md)
- [Runtime Architecture](specification/runtime_architecture.md)
