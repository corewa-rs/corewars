# Redcode Syntax highlighting

[![Latest Release](https://img.shields.io/visual-studio-marketplace/v/ian-h-chamberlain.redcode?logo=visual-studio-code)](https://marketplace.visualstudio.com/items?itemName=ian-h-chamberlain.redcode)
[![Build Status](https://img.shields.io/github/workflow/status/ian-h-chamberlain/corewa_rs/ci/master)](https://github.com/ian-h-chamberlain/corewa_rs/actions)

Basic syntax highlighting for Redcode, the language used in [Core Wars](https://corewa.rs).

## Features

Syntax highlighting for redcode, based partially on [this vim syntax file](https://www.vim.org/scripts/script.php?script_id=1705), and on the [pMARS '94 reference](https://corewa.rs/pmars-redcode-94.txt) and an [annotated version](https://corewa.rs/icws94.txt) of the ICWS '94 draft.

## Contents

* `package.json` - NPM package declaration for the extension
* `package-lock.json` - pinned NPM dependencies
* `syntaxes/redcode.tmLanguage.yaml` - the TextMate grammar definition, maintained for readability and editability
* `syntaxes/redcode.tmLanguage.json` - the TextMate grammar file used for tokenization, generated from `redcode.tmLanguage.yaml`.
* `language-configuration.json` - this the language configuration, defining the tokens that are used for comments and brackets

## Release Notes

See [the changelog](./CHANGELOG.md).
