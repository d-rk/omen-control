# fan-control Specification

## Purpose

Lets the user watch fan speeds and temperatures and set the thermal profile, max fan, custom fan curves and CPU power
limits through the driver's fan interface.

## Requirements

### Requirement: Live readings
The fans page SHALL show CPU and GPU fan RPM and CPU and GPU temperature, refreshed about once a second, and the ACPI
platform profile as read-only text. An unavailable reading SHALL show as "n/a".

#### Scenario: Unavailable RPM shown as n/a
- **WHEN** `gpu_fan_rpm` reads `n/a`
- **THEN** the GPU RPM shows "n/a"

### Requirement: Thermal profile
The page SHALL show `silent`, `normal` and `performance` as segmented buttons with the current one highlighted, and
pressing one SHALL write it to `thermal_profile`.

#### Scenario: Select thermal profile
- **WHEN** the user presses "performance"
- **THEN** `performance` is written to `thermal_profile`

### Requirement: Fan switches
The page SHALL offer toggles for `max_fan` and `fan_curve_enable` that write `1` or `0`.

#### Scenario: Toggle max fan
- **WHEN** the user turns max fan on
- **THEN** `1` is written to `max_fan`

### Requirement: Curve editor
The page SHALL draw the CPU or GPU curve (chosen by tabs) on a chart with temperature 30-100 °C across and fan 0-100 %
up. The user SHALL be able to drag a point, add a point by clicking empty space, and remove a point by right-clicking
it. A dragged point SHALL stay inside the chart and between its neighbours' temperatures, and the curve SHALL keep 2 to
8 points.

#### Scenario: Drag point clamped to chart
- **WHEN** the user drags a point above the top of the chart
- **THEN** the point's percent is 100

#### Scenario: Drag point cannot pass neighbour
- **WHEN** the user drags a point to the left past the previous point
- **THEN** the point's temperature stops one degree above the previous point's

#### Scenario: Add point
- **WHEN** a curve has fewer than 8 points and the user clicks empty chart space
- **THEN** a point is added there, in temperature order

#### Scenario: Add point at limit
- **WHEN** a curve has 8 points and the user clicks empty chart space
- **THEN** no point is added

#### Scenario: Remove point
- **WHEN** a curve has more than 2 points and the user right-clicks a point
- **THEN** that point is removed

#### Scenario: Remove point at minimum
- **WHEN** a curve has 2 points and the user right-clicks a point
- **THEN** no point is removed

### Requirement: Apply and revert curves
Curve edits SHALL stay local until the user presses "Apply", which SHALL write the CPU curve to `fan_curve` and either
the GPU curve to `gpu_fan_curve` or `cpu` when "GPU follows CPU" is checked. "Revert" SHALL reload both curves from the
driver.

#### Scenario: Apply with GPU following CPU
- **WHEN** "GPU follows CPU" is checked and the user presses "Apply"
- **THEN** the CPU curve is written to `fan_curve` and `cpu` is written to `gpu_fan_curve`

#### Scenario: Revert discards edits
- **WHEN** the user has moved a point and presses "Revert"
- **THEN** the curve shows the points read from the driver

### Requirement: Power limits
The page SHALL offer PL1 and PL2 inputs in watts with "Apply" and "Reset to default". Invalid input SHALL show an inline
error and write nothing; "Reset to default" SHALL write `default`.

#### Scenario: Invalid power limits not written
- **WHEN** the user enters PL1 90 and PL2 65 and presses "Apply"
- **THEN** nothing is written and an inline error says PL2 must be at least PL1

#### Scenario: Reset power limits
- **WHEN** the user presses "Reset to default"
- **THEN** `default` is written to `power_limits`

### Requirement: Missing fan controls
A fan control whose sysfs file is missing SHALL be hidden or disabled, and a write that fails SHALL show its error
inline on the page.

#### Scenario: Power limits unsupported
- **WHEN** `power_limits` reads `unsupported`
- **THEN** the power limit inputs are disabled
