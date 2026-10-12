# Workshop Walker

[![Release](https://img.shields.io/github/v/release/disconsented/workshop-walker?sort=semver)](https://github.com/disconsented/workshop-walker/releases)
[![rustfmt](https://img.shields.io/github/actions/workflow/status/disconsented/workshop-walker/rustfmt.yml?branch=master&label=rustfmt)](https://github.com/disconsented/workshop-walker/actions/workflows/rustfmt.yml)
[![ui](https://img.shields.io/github/actions/workflow/status/disconsented/workshop-walker/ui.yml?branch=master&label=ui)](https://github.com/disconsented/workshop-walker/actions/workflows/ui.yml)
[![License: MPL-2.0](https://img.shields.io/github/license/disconsented/workshop-walker)](LICENSE.md)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-fe5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)

---

## About

Workshop Walker is an alternate interface to the Steam Workshop, making it easier to find the items you care about.

It does this by taking the data freely provided by Valve, analysing it and serving it up in a tidy package.

Specific improvements:

- Dependant discovery, find everything that depends on this item.
- Language detection, filter for specific languages. No more poorly marked translations.
- Properties, extended classification, submit new ones or vote on existing entries for relevance.

Interested? You can check it out live at https://workshop-walker.disconsented.com/

## Screenshots

### Search

![Search Page](docs/search.png)

### Item Page

![Dead Mans Switch Item Page](docs/item-dms.png)

## How it works

Workshop Walker pulls every item from the Steam Workshop periodically via their API, processing each item individually for the following:

- Language detection on the item's body.
- Converting BB Code to HTML for rendering.
- ML assessments for properties if the description has changed.
- Inserting relationships between items. This is something
  that [SurrealDB lets us model cleanly](https://surrealdb.com/docs/reference/query-language/statements/relate).

## Suggest a Game

This project was created for RimWorld originally, however it is designed to be agnostic and work with any Steam Workshop
game.

If you think that it'd be useful to include, please
open [a new discussion](https://github.com/disconsented/workshop-walker/discussions/new?category=game-suggestions).

Please do take care to explain how Workshop Walker will be useful, I'd rather not add games that won't benefit or won't get used.

## Contributing

Want to contribute a feature? Great!

Please suggest it as an _issue_ to get a feeling on whether I'll accept it. Remember, that every piece of code
contributed adds to the maintenance burden. This is also an effort to manage burnout.

For anything touching the UI, ideally include some kind of visual reference.

If it's accepted:

- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/). release-plz builds the changelog and version from them.
- Rust: `cargo +nightly fmt --all` and `cargo clippy --all` must pass. Clippy warnings fail the build.
- UI: `npm run format`, `npm run lint` and `npm run check` from `ui/` must pass.

--- 

This project is not associated with Valve, Steam, etc. I just got annoyed one day I couldn't see dependants and wanted an
excuse to use SurrealDB relations.