noten, the NOstressz TEmplating ENgine.

Written with my mother's website in mind, but trying to be as generic
as possible.

# Overview
The configuration file is `noten.toml`.
Noten looks for this in the current directory.
If it's not found, then the current directory is not a valid noten project.

# Configuration file format
The configuration file is written in the TOML format.
Here is a listing of the options.

## [directories]
These are the various directories the noten works with.

name       | desc
---------- | ---
input      | The directory the templates are read from.
output     | The directory that the output is written to.
generators | (Optional) The directory where generators are located.

## [constants]
You can define various constants here that you can use in your templates.
Any type that TOML accepts is valid.
