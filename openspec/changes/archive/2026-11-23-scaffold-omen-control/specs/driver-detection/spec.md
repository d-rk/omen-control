# Spec Delta

## Purpose

Tells the user when the omen-rgb-keyboard driver is not available and how to install and enable it, instead of
showing empty controls.

## ADDED Requirements

### Requirement: Driver presence
The driver SHALL count as present when the sysfs root contains an `rgb_zones` or a `fan` directory.

#### Scenario: Root missing
- **WHEN** the sysfs root does not exist
- **THEN** the driver is reported as missing

#### Scenario: Only fan directory
- **WHEN** the root contains `fan/` but no `rgb_zones/`
- **THEN** the driver is reported as present

### Requirement: Driver not found screen
When the driver is missing the app SHALL show a screen instead of the pages that explains: the omen-rgb-keyboard DKMS
driver is needed, the udev rules and `input` group step, blacklisting `hp_wmi`, and module signing under Secure Boot.
It SHALL offer a "Check again" button.

#### Scenario: Check again finds driver
- **WHEN** the screen is shown, the driver appears, and the user presses "Check again"
- **THEN** the app shows the pages

### Requirement: Secure Boot hint
If `mokutil --sb-state` can be run, the screen SHALL show whether Secure Boot is enabled; if it cannot, the screen SHALL
show the generic signing hint only.

#### Scenario: Parse Secure Boot state
- **WHEN** `mokutil --sb-state` prints `SecureBoot enabled`
- **THEN** Secure Boot is reported as enabled

### Requirement: Copyable setup commands
Each shell command on the driver-not-found screen SHALL have a "Copy" button that puts the command's exact text on the
clipboard and confirms the copy in the status bar.

#### Scenario: Copy setup command
- **WHEN** the user presses "Copy" next to the udev rules command
- **THEN** the clipboard holds `sudo ./install-udev-rules.sh` and the status bar says the command was copied
