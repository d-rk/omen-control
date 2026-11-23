# sysfs-backend Specification

## Purpose

Gives the rest of the app typed access to the omen-rgb-keyboard driver's sysfs files, the thermal zones and the ACPI
platform profile, in the exact text formats the driver reads and writes.

## Requirements

### Requirement: Configurable sysfs root
The app SHALL read the driver from `/sys/devices/platform/omen-rgb-keyboard` unless the `OMEN_SYSFS_ROOT` environment
variable names another directory, in which case it SHALL use that directory.

#### Scenario: Default root
- **WHEN** `OMEN_SYSFS_ROOT` is unset and no `--mock` flag is given
- **THEN** the root is `/sys/devices/platform/omen-rgb-keyboard`

#### Scenario: Root from environment
- **WHEN** `OMEN_SYSFS_ROOT` is set to a directory path
- **THEN** the root is that path

### Requirement: Mock tree
With `--mock`, the app SHALL create a temporary directory holding every `rgb_zones/` and `fan/` file with plausible
default values, plus a fake thermal zone and platform profile, and SHALL use it as the root.

#### Scenario: Mock tree is readable as a driver
- **WHEN** the mock tree is created and the lighting and fan state is read from it
- **THEN** every value parses and the driver is reported as present

### Requirement: Colour values
Zone colours SHALL be read in the driver's `#rrggbb` form and also accepted as bare `RRGGBB` in either case, and SHALL
be written as six uppercase hex digits.

#### Scenario: Parse driver colour output
- **WHEN** a zone file contains `#ff8000\n`
- **THEN** it parses as red 255, green 128, blue 0

#### Scenario: Reject malformed colour
- **WHEN** a colour string is not six hex digits after an optional `#`
- **THEN** parsing fails with an error naming the bad value

#### Scenario: Format colour for writing
- **WHEN** the colour red 255, green 128, blue 0 is written
- **THEN** the text written is `FF8000`

### Requirement: Fan curve values
A fan curve SHALL be read from space-separated `temp:percent` pairs, where the percent may be `max` (meaning 100).
`(unset)` SHALL read as no curve, and for the GPU curve `(cpu)` SHALL read as following the CPU curve. A curve SHALL be
written as its points sorted by temperature, and only when it has 2 to 8 points with temperatures 0-120 and percents
0-100.

#### Scenario: Parse curve
- **WHEN** a curve file contains `45:0 60:40 75:70 92:100\n`
- **THEN** it parses as four points in that order

#### Scenario: Parse max keyword
- **WHEN** a curve point is `92:max`
- **THEN** its percent is 100

#### Scenario: Parse GPU following CPU
- **WHEN** the GPU curve file contains `(cpu)`
- **THEN** the GPU curve reads as following the CPU curve

#### Scenario: Parse unset curve
- **WHEN** a curve file contains `(unset)`
- **THEN** the curve reads as having no points

#### Scenario: Format curve sorted
- **WHEN** a curve with points 75:70, 45:0 and 60:40 is formatted for writing
- **THEN** the text is `45:0 60:40 75:70`

#### Scenario: Reject curve with too few or too many points
- **WHEN** a curve with 1 point or with 9 points is validated
- **THEN** validation fails

#### Scenario: Reject curve point out of range
- **WHEN** a curve has a point with percent above 100 or temperature above 120
- **THEN** validation fails

### Requirement: Gradient configuration values
`gradient_config` SHALL be read and written as `;`-separated groups of `zones:colours`, where zones are a
`,`-separated list of 0-3 and colours a `,`-separated list of hex colours. A group SHALL need at least one zone and one
colour.

#### Scenario: Parse gradient config
- **WHEN** the file contains `0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500`
- **THEN** it parses as two groups: zones 0,1,2 with three colours, and zone 3 with two colours

#### Scenario: Gradient config round trip
- **WHEN** a parsed gradient config is formatted again
- **THEN** the text equals the driver's format, `0,1,2:FF0000,00FF00,0000FF;3:800080,FFA500`

#### Scenario: Reject gradient zone out of range
- **WHEN** a group names zone 4
- **THEN** parsing fails

### Requirement: Power limit values
`power_limits` SHALL read `default` as the BIOS defaults, `unsupported` as not available, and `PL1 PL2` as two wattages.
A custom pair SHALL be written only when both are positive and PL2 is at least PL1.

#### Scenario: Parse power limits
- **WHEN** the file contains `65 90`
- **THEN** it parses as PL1 65 W and PL2 90 W

#### Scenario: Parse default power limits
- **WHEN** the file contains `default`
- **THEN** it parses as the BIOS defaults

#### Scenario: Reject PL2 below PL1
- **WHEN** PL1 is 90 and PL2 is 65
- **THEN** validation fails with an error saying PL2 must be at least PL1

### Requirement: Fan readings
`cpu_fan_rpm` and `gpu_fan_rpm` SHALL read `n/a` as not available. `max_rpm` SHALL read `CPU GPU` as two maximums and
`unknown` as not available.

#### Scenario: Parse unavailable RPM
- **WHEN** an RPM file contains `n/a`
- **THEN** the reading is not available

#### Scenario: Parse max RPM
- **WHEN** `max_rpm` contains `5200 5400`
- **THEN** the CPU maximum is 5200 and the GPU maximum is 5400

### Requirement: Temperatures from thermal zones
The app SHALL find a thermal zone by its `type` under the thermal class directory and read its `temp` in millidegrees as
°C. A temperature zone setting of `(auto)` SHALL use the first of `x86_pkg_temp`, `coretemp`, `k10temp`, `acpitz` that
exists; `(cpu)` for the GPU SHALL use the CPU zone.

#### Scenario: Read zone temperature
- **WHEN** a thermal zone with type `x86_pkg_temp` has `temp` `54000`
- **THEN** the CPU temperature reads as 54.0 °C

### Requirement: Optional files
Every sysfs file SHALL be optional. A missing file SHALL be reported as unavailable rather than as an error, and a
failed write SHALL return an error naming the file and the cause.

#### Scenario: Missing file is unavailable
- **WHEN** the `fan/` directory has no `gpu_fan_rpm`
- **THEN** the GPU RPM reads as unavailable and the other fan values still read

#### Scenario: Failed write reports file
- **WHEN** writing a file fails
- **THEN** the error names that file

### Requirement: Platform profile
The app SHALL read `/sys/firmware/acpi/platform_profile` (or `firmware/platform_profile` under the mock root) and
treat it as read-only.

#### Scenario: Platform profile shown
- **WHEN** the platform profile file contains `balanced`
- **THEN** the platform profile reads as `balanced`
