# yRice
## Introduction
**yRice** is a dotfile management helper written in rust based on a _yaml_ configuration file.

## Installation
Clone this repository and run `sudo cargo install -path .`. 

This will install the binary to cargo's `bin` dir, which you need to add to your `$PATH` 
(or you can manually move the binary to any directory in your `$PATH`).

## Basic usage
```bash
$ yrice -h
yrice 0.1.0

USAGE:
    yrice [OPTIONS] [MODULES]...

ARGS:
    <MODULES>...    

OPTIONS:
    -c, --config <CONFIG>    
    -d, --dry-run            
    -h, --help               Print help information
    -i, --install            
    -V, --version            Print version information
```
If using the `-i` flag, **yRice** will install the system packages. See below for details.

## Configuration
**yRice** will get its configuration either from the path specified using the `-c`flag or `$XDG_CONFIG_DIR/yrice/config.yaml` 
(falling back to `~/.config/yrice/config.yaml`).

An example configuration can be found [here](./config.example.yaml). A more complete example is in [my dotfiles](https://github.com/LoricAndre/dotfiles/blob/main/yrice/config.yaml).

### Global
The `global` key is used to configure yRice itself :
- `installCommand` will be used to install packages, for example using your system's package manager (`apt`, `yum`, `pacman`...)
- `dotfiles` will tell **yRice** where to find the source configuration files for all the specified modules
### Variables
The `variables` list will be used when a module file has `parse` set to `true` (see below for more detail). 
The parsing is done using [handlebars](https://handlebarsjs.com/guide/), simply replacing anything between `{{...}}` with the variable content when enabled.
### Modules
Modules are the heart of **yRice**. They define which programs are managed using **yRice**.
For each module, **yRice** will look for a folder named `dirname` (defaulting to the module name) in the `dotfiles` dir, 
then try to install all specified files to `targetDir` (defaulting to `$XDG_CONFIG_DIR/dirname` or `~/.config/dirname`, 
with `dirname` defaulting to the module name).

If a file has `parse` set to `true`, **yRice** will parse the file and _write it_ to the specified location, after having created a backup of the original file to `targetDir/target.bak`.
If it is unset (or set to `false`), the file will be _softlinked_ and **no backup will be created**. Be careful when using this, as any current configuration files will be overwritten.

`preSteps` and `postSteps` will be run in a shell before or after the file was linked or written.

*Examples:* 
- Using the default module configuration `yrice: {}`, **yRice** will:
  1. look for a folder named `yrice` in `dotfiles`
  2. softlink this folder to `~/.config/yrice`
- With the following config, **yRice** will:
  ```
  foo:
   files:
   - source: bar
     target: baz
     parse: true
  ```
  1. look for a file named `foo/bar` in `dotfiles`
  2. replace any handlebars found in the file with the associated variable's content
  3. write the result to `~/.config/foo/baz`
