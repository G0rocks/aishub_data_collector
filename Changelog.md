# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-

### Fixed

- 

### Changed

- 

### Removed

- 

## [2.0.0] - 2026-02-04

### Added
- ship.csv file now also has notes

### Changed
- Now saves data in csv files with semicomma separated values instead of comma separated values. This means all csv files use semicomma delimiters now.
- Replaced the vec_to_comma_separated_string function with vec_to_delimiter_separated_string function which allows custom delimiters to be set.

## [v1.1.0] - 2025-11-27

### Added

- Added vessel names to vessel files, see issue #7
- Added all setting parameters to make_aishub_url() function.
- Added a list of invalid filename characters
- Added a new function, make_filename(), which uses the list of invalid filename characters and replaces them with an underscore in case they show up in the vessel name. See issue #8

### Fixed

- Bug where some filenames could not be saved, see issue #8

### Changed

- Cleaned up the code, some fields were named ABC_min and others min_ABC, now always ABC_min or ABC_max.
- Filenames now include name of vessel
- Handle most errors without crashing instead of panicking, see issue #3
- settings_example.json settings are now in alphabetical order
- Format logs in a more readable format, see issue #5
- Now uses previously used settings if unable to read current settings so it can only crash if it does not work initially which is great. See issue #8
- Improved error message when problems with saving data. See issue #8

## [v1.0.0] - 2025-10-23
The first release is out!
We have some structs and enums and the capability to download data from AISHub to csv files continuously.
Glad to be here :)

### Added

- main.rs

### Fixed

- Nothing was fixed

### Changed

- Nothing was changed

### Removed

- Nothing was removed

## List of releases
[unreleased]: https://github.com/G0rocks/marine_vessel_simulator/compare/2.0.0...main
[2.0.0]: https://github.com/G0rocks/marine_vessel_simulator/releases/tag/2.0.0
[1.1.0]: https://github.com/G0rocks/marine_vessel_simulator/releases/tag/v1.1.0
[1.0.0]: https://github.com/G0rocks/marine_vessel_simulator/releases/tag/v1.0.0