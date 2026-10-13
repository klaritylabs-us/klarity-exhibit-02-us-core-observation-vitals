# US Core vital signs Observation (FHIR R4) — Rust

Small library and CLI that sanity-check **US Core vital sign** `Observation` JSON before it leaves your integration layer: blood pressure **components**, **vital-signs** category, **UCUM** units, and **LOINC** on the observation `code`.

Synthetic fixtures only. Not ONC certification, not a substitute for the HL7 FHIR Validator or Inferno, and not legal/clinical advice.

Maintained by [Klarity Labs](https://klaritylabs.ai/) as open reference code for U.S. healthcare FHIR interoperability work.

**Exhibit 02** — [FHIR exhibit catalog](https://github.com/klaritylabs-us/klarity-fhir-exhibits) · [klaritylabs-us](https://github.com/klaritylabs-us)


## How this relates to other tools

| Tool class | Role |
|------------|------|
| **HL7 FHIR Validator**, **HAPI**, **Firely** | Full validation, servers, SDKs, IG packages |
| **Inferno**, **Touchstone**, vendor cert suites | **Conformance** against test plans |
| **This exhibit** | **Pre-flight construction** on synthetic fixtures—catch shape mistakes before wire traffic |

Conformance tools test whether your **API behaves**; exhibits focus on whether your **payloads and pipes are built** to survive real partners.

## Specifications cited

- [US Core Observation](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-observation.html)
- [Vital signs profile](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-vital-signs.html)

## What it checks

- Blood pressure uses **components** (systolic `8480-6`, diastolic `8462-4`, panel `85354-9`) — not a single parent `valueQuantity`
- Category includes `http://terminology.hl7.org/CodeSystem/observation-category|vital-signs`
- `valueQuantity.system` is UCUM (`http://unitsofmeasure.org`)
- Vital `code` LOINC matches the metric (e.g. heart rate vs body weight mix-ups)

| Problem | What goes wrong |
|--------|------------------|
| Blood pressure shape | One value on the parent instead of component Observations |
| Category | Vitals copied from lab Observations without vital-signs category |
| UCUM | `mm[Hg]`, `/min`, or other units on the wrong system URI |
| Wrong LOINC | Plausible numeric value with the wrong observation code |

## Profiles covered

- [US Core Blood Pressure](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-blood-pressure.html)
- [US Core Heart Rate](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-heart-rate.html)
- [US Core Body Weight](http://hl7.org/fhir/us/core/STU6.1/StructureDefinition-us-core-body-weight.html)

## Run it

Needs [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo test
cargo run --bin validate-observation -- examples/valid-blood-pressure.json
cargo run --bin validate-observation -- examples/invalid-heart-rate-wrong-loinc.json
```

Exit `0` if no errors, `1` if validation failed, `2` for bad usage or JSON.

## Sample output

One command:

```bash
cargo run --bin validate-observation -- examples/valid-blood-pressure.json
```

**Pass** (`examples/valid-blood-pressure.json`) — exit code `0`:

```json
[]
```

**Fail** (`examples/invalid-bp-single-value.json`) — exit code `1` (issues array; more entries may appear):

```json
[
  {
    "severity": "error",
    "path": "Observation.valueQuantity",
    "detail": "blood pressure uses component observations, not valueQuantity on the parent",
    "rule_id": "bp-no-parent-value"
  },
  {
    "severity": "error",
    "path": "Observation.component",
    "detail": "blood pressure requires component entries for systolic and diastolic",
    "rule_id": "bp-components-required"
  }
]
```


## Layout

- `src/validate.rs` — rules
- `examples/` — valid and broken payloads for tests and demos
- [docs/architecture.md](./docs/architecture.md) — short pipeline note

## Inferno vs this repo

Inferno exercises your **API** against CMS-style scenarios. This repo is about **payload shape** — vitals that still break analytics and quality programs after a green conformance run.

## License

MIT — see [LICENSE](./LICENSE).

**Need this on your payloads or API?** 20-minute FHIR Scan — sk@klaritylabs.ai · [klaritylabs.ai](https://klaritylabs.ai/)

## Contact

sk@klaritylabs.ai · [klaritylabs.ai](https://klaritylabs.ai/)
