# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Windows service support with [windows-service-rs](https://github.com/mullvad/windows-service-rs)
- Added Powershell scripts to allow easy service installation (Powershell 6 or greater is required)

### Changed
- Moved windows specific functions to own module
- Moved service name definition to external `service_name.in` file to allow both powershell scripts and rust code to have matching definition
- Changed from [winlog](https://crates.io/crate/winlog) to my own EventLog adapter [windows-event-log](https://github.com/kallekankaanpaa/windows-event-log)

## [0.0.1] - 2022-05-28
### Added
- Base for the project. Nothings really working yet but the basic structure for the project is defined. 