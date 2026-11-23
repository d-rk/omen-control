# Spec Delta

## Purpose

Lets the user save the current lighting (and optionally fan) settings under a name and switch back to them later in
one step.

## ADDED Requirements

### Requirement: Profile storage
Profiles SHALL be stored as one TOML file per profile in `omen-control/profiles/` under the user's config directory.
A profile holds the lighting settings and, if the user chose to include them, the fan settings.

#### Scenario: Save and load round trip
- **WHEN** a profile with lighting and fan settings is saved and then loaded
- **THEN** the loaded profile equals the saved one

#### Scenario: List profiles sorted
- **WHEN** profiles "Work", "gaming" and "Night" are saved
- **THEN** the profile list is "gaming", "Night", "Work" (case-insensitive order)

#### Scenario: Delete profile
- **WHEN** a saved profile is deleted
- **THEN** it no longer appears in the profile list

### Requirement: Profile names
A profile name SHALL be 1 to 64 characters after trimming and SHALL NOT contain `/`, `\` or control characters, or
start with `.`. Saving under an existing name SHALL replace that profile.

#### Scenario: Reject path in name
- **WHEN** the user saves a profile named `../evil`
- **THEN** saving fails with an error about the name and no file is written

### Requirement: Applying a profile
Applying a profile SHALL write its lighting settings to the driver, and its fan settings only if the profile includes
them. A field whose file is missing SHALL be skipped; write errors SHALL be collected and shown, not stop the rest.

#### Scenario: Profile without fan settings leaves fans alone
- **WHEN** a profile without fan settings is applied
- **THEN** no file under `fan/` is written

#### Scenario: Apply continues after a failed write
- **WHEN** writing one lighting value fails while applying a profile
- **THEN** the other values are still written and the error is reported

### Requirement: Header quick selector
The window header SHALL show a profile selector listing saved profiles; choosing one SHALL apply it.

#### Scenario: Choose profile from header
- **WHEN** the user chooses "gaming" in the header selector
- **THEN** the "gaming" profile is applied
