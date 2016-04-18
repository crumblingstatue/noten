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

## skeleton
The path of the [skeleton template](#the-skeleton-template).

## index
The name of the document that will be treated as the index.
A copy of it will be stored as `index.html`.

## [directories]
These are the various directories the noten works with.

name       | desc
---------- | ----
input      | The directory the templates are read from.
output     | The directory that the output is written to.
generators | (Optional) The directory where generators are located.

## [constants]
You can define various constants here that you can use in your templates.
Any type that TOML accepts is valid.

# The skeleton template
The skeleton template is used as the skeleton for generating the output for each page.
It exists because a website usually has a basic skeleton that is the same
for all pages (e.g. the header, the menu, etc), and should not be repeated
manually. It has a few special powers that are not available in normal templates.

## Special substitution commands
It has the following special substitution commands that are not available in normal
templates.

name         | desc
------------ | ----
title        | Title of the child template
description  | Description of the child template (optional)
content      | The content of the child template
if-exists    | Used for handling optional substitution commands

### if-exists
`if-exists <name> <template-string>`

It checks if the substitution command with `<name>` exists, if it does, it
emits the template `<template-string>`, otherwise, it does nothing.

If a template string is emitted, it is then expanded like any other template.

# Processing the templates
Noten reads each template in the `directories.input` directory, processes them,
and outputs the generated documents to `directories.output`.
It only processes files with the extension `.noten`.

## Template syntax
### Attribute list
Each template optionally begins with an attribute list.
An attribute list begins with `{` and ends with `}`.
In between the curly braces, it contains various attributes of the document
in TOML format.

Here are some attributes you can define:

name        | desc
----------- | ----
title       | The title of the page. If not given, it will be computed according to [Title computation](#title-computation).
description | The html meta description of the page.

You can also declare constants in the attribute list.
Constants declared here shadow global constants.

#### Title computation

If no title is given in the attribute list, it will be computed like this:
The first non-empty line must either be a markdown or HTML header, and its content
will be used as the title. If it does not satisfy this requirement, then the document is not
a valid noten template.

### Substitution
In addition to just normal text that gets interpreted as-is, templates can
contain substitutions, which get replaced by the thing they describe.
Each substitution begins with `{{` and ends with `}}`.
