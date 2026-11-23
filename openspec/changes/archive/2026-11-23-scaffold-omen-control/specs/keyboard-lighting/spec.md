# Spec Delta

## Purpose

Lets the user set the keyboard's four lighting zones, brightness and animation from a visual keyboard preview instead
of writing sysfs files by hand.

## ADDED Requirements

### Requirement: Zone selection
The lighting page SHALL draw the keyboard split into its four zones, each filled with its colour. Which keys a zone
lights is decided by the laptop's firmware, so the page SHALL pick the layout from the DMI product name
(`/sys/class/dmi/id/product_name`):

- OMEN Transcend 14 (product name contains `14-fb0`): a compact layout without a numpad. W, A, S and D are zone 3;
  the remaining keys are split into zone 2 (left), zone 1 (middle) and zone 0 (right).
- Any other or unreadable product name: a full layout with a numpad, split left to right into zone 0 to zone 3.

Clicking a zone SHALL select it and highlight it; exactly one zone SHALL be selected at a time.

#### Scenario: Click selects zone
- **WHEN** the user clicks inside the area of zone 2
- **THEN** zone 2 becomes the selected zone

#### Scenario: Transcend 14 zone layout
- **WHEN** the product name is `OMEN Transcend Gaming Laptop 14-fb0xxx`
- **THEN** the keyboard has no numpad, W, A, S and D are in zone 3, Q is in zone 2, L is in zone 1 and P is in zone 0

#### Scenario: Unknown model uses the full layout
- **WHEN** the product name is unknown
- **THEN** the keyboard has a numpad and its zones run left to right from zone 0 to zone 3

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

### Requirement: LED colour correction
The keyboard drives its LEDs linearly with the values in the colour files, while the app shows colours as a screen
does. Colours SHALL be converted with a gamma of 2.8 per channel (`led = 255 × (screen / 255)^2.8`, rounded) when
written to `zone00`–`zone03`, `all` and `gradient_config`, and converted back when read, so the keyboard matches the
colour shown in the app. Black and full-intensity channels are unchanged.

#### Scenario: Colour gamma-corrected on write
- **WHEN** the user sets zone 0 to `E4002B`
- **THEN** `BA0002` is written to `zone00`

#### Scenario: Corrected colour read back
- **WHEN** `zone00` contains `BA0002`
- **THEN** zone 0 shows a colour that is written back as `BA0002`

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
