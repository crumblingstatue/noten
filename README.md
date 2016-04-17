noten, the NOstressz Templating ENgine.

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

# Processing the templates
Noten reads each template in the `directories.input` directory, processes them,
and outputs the generated documents to `directories.output`.

## Template syntax
### Attribute list
Each template optionally begins with an attribute list.
An attribute list begins with `{` and ends with `}`.
In between the curly braces, it contains various attributes of the document
in TOML format.

You can also declare constants in the attribute list.
Constants declared here shadow global constants.
### Substitution
In addition to just normal text that gets interpreted as-sis, templates can
contain substitutions, which get replaced by the thing they describe.
Each substitution begins with `{{` and ends with `}}`.
