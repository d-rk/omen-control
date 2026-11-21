# Spec Delta

## Purpose

Lets the user set the keyboard's four lighting zones, brightness and animation from a visual keyboard preview instead
of writing sysfs files by hand.

## ADDED Requirements

### Requirement: Zone selection
The lighting page SHALL draw the keyboard split into four zones, left to right zone 0 to zone 3, each filled with its
colour. Clicking a zone SHALL select it and highlight it; exactly one zone SHALL be selected at a time.

#### Scenario: Click selects zone
- **WHEN** the user clicks inside the area of zone 2
- **THEN** zone 2 becomes the selected zone

#### Scenario: Click outside keyboard keeps selection
- **WHEN** the user clicks outside the keyboard outline
- **THEN** the selected zone does not change

### Requirement: Zone colour
Picking a colour SHALL write it to the selected zone's file at once. "Apply to all" SHALL write the selected zone's
colour to the `all` file and update all four zones in the preview.

#### Scenario: Pick colour writes selected zone
- **WHEN** zone 1 is selected and the user picks `00FF00`
- **THEN** `00FF00` is written to `zone01` and zone 1 shows green

#### Scenario: Apply to all
- **WHEN** the selected zone is red and the user presses "Apply to all"
- **THEN** `FF0000` is written to `all` and every zone shows red

### Requirement: Colour picker
The colour picker SHALL offer a saturation/value square with a hue bar, a hex input and a row of swatches. Hex input
SHALL accept six hex digits with or without `#` and SHALL show an inline error otherwise.

#### Scenario: HSV round trip
- **WHEN** a colour is converted to hue, saturation and value and back
- **THEN** each channel differs from the original by at most 1

#### Scenario: Invalid hex input
- **WHEN** the user types `12345G` in the hex input
- **THEN** no colour is written and an inline error is shown

### Requirement: Brightness and speed
The page SHALL offer a brightness slider 0-100 and an animation speed slider 1-10 that write `brightness` and
`animation_speed`.

#### Scenario: Brightness written
- **WHEN** the user sets brightness to 40
- **THEN** `40` is written to `brightness`

### Requirement: Animation mode
The page SHALL offer the twelve driver animation modes and write the chosen one to `animation_mode`. The gradient editor
SHALL be shown only while the mode is `gradient`.

#### Scenario: Gradient editor only in gradient mode
- **WHEN** the mode is changed from `gradient` to `wave`
- **THEN** the gradient editor is hidden

### Requirement: Gradient editor
The gradient editor SHALL let the user add and remove groups, toggle which zones a group covers and edit its colour
list, and SHALL write the result to `gradient_config` only when every group has at least one zone and one colour.

#### Scenario: Incomplete gradient group not written
- **WHEN** a gradient group has no zones and the user applies the gradient
- **THEN** nothing is written and an inline error names the group

### Requirement: Animation preview
The keyboard preview SHALL animate an approximation of the selected mode at the selected speed, scaled by brightness.
The preview is approximate and SHALL NOT be relied on to match the hardware frame for frame.

#### Scenario: Static preview shows zone colours
- **WHEN** the mode is `static` and brightness is 100
- **THEN** each zone in the preview shows its configured colour

#### Scenario: Breathing preview dims over time
- **WHEN** the mode is `breathing`
- **THEN** the preview colour of a zone is dimmer at some point in the cycle than at its peak

### Requirement: Missing lighting controls
A lighting control whose sysfs file is missing SHALL be hidden or disabled, and a write that fails SHALL show its
error inline on the page.

#### Scenario: Write error shown
- **WHEN** writing `brightness` fails
- **THEN** the page shows the error and the slider keeps the last value read from the driver
