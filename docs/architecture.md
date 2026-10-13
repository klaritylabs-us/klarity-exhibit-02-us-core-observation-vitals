# Architecture

```text
Observation.json
  → parse
  → profile branch (BP | heart rate | body weight from meta.profile)
  → vital-signs category + LOINC on code
  → profile-specific:
        BP: panel LOINC, components, no parent valueQuantity, mm[Hg] + UCUM
        HR / weight: valueQuantity, LOINC/code alignment, UCUM
  → ValidationIssue list (JSON from CLI)
```

Out of scope: full StructureDefinition validation, terminology server calls, reference integrity to real Patient servers.
