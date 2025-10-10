# Cli Reference

This document contains the help content for the `` command-line program.

**Command Overview:**

* [``↴](#)
* [` generate`↴](#-generate)
* [` check`↴](#-check)
* [` init`↴](#-init)
* [` fmt`↴](#-fmt)

## ``

**Usage:** `[OPTIONS] <COMMAND>`

###### **Subcommands:**

* `generate` — generates models as defined in `op-gen.toml`
* `check` — checks models for soundness
* `init` — initializes a new schema project
* `fmt` — formats schemas

###### **Options:**

* `--log-level <LOG_LEVEL>` — the verbosity level to print logs at.

  Default value: `info`

  Possible values: `debug`, `trace`, `info`, `error`, `warn`




## ` generate`

generates models as defined in `op-gen.toml`

**Usage:** ` generate [OPTIONS]`

###### **Options:**

* `-d`, `--config-dir <CONFIG_DIR>`



## ` check`

checks models for soundness

**Usage:** ` check [OPTIONS]`

###### **Options:**

* `-d`, `--config-dir <CONFIG_DIR>`



## ` init`

initializes a new schema project

**Usage:** ` init [OPTIONS] --name <NAME>`

###### **Options:**

* `-n`, `--name <NAME>` — the name of the package to create.
* `-d`, `--dir <DIR>` — the directory to create the new package in.



## ` fmt`

formats schemas

**Usage:** ` fmt [OPTIONS] [INCLUDE]...`

###### **Arguments:**

* `<INCLUDE>` — a list of paths or globs to include in formatting

  Default value: `./**/*.pld`

###### **Options:**

* `-d`, `--config-dir <CONFIG_DIR>`
* `--dry` — if --dry, no edits will be written to files

  Default value: `false`
* `--safe` — if --safe=false, unsafe edits will be applied

  Default value: `true`
* `-e`, `--exclude <EXCLUDE>` — a list of paths or globs to exclude from formatting.
* `-W`, `--warn-is-fail` — fail if warnings are encountered
