## Workshop Walker

Workshop Walker is an alternate interface to the steam workshop, improving discoverability and filtering of items on the steam workshop.

Specific improvements:

- Dependant discovery, find everything that depends on this item
- Language detection, filter for specific languages. No more poorly marked translations
- Properties, extended classification, submit new ones or vote on existing entries for relevance 

A live version of this project can be found at https://workshop-walker.disconsented.com/

## How it works

Workshop Walker (WW) makes
use of [SurrealDB's relationship modeling](https://surrealdb.com/docs/surrealql/datamodel/ids), where the dependency
relationships can be efficiently reversed. Which gives us cheap dependant lookups.

Language support is handled heuristically by https://crates.io/crates/lingua

Everything else is pretty straight forward.

## Suggest a Game/Tag

Please put requests/suggestions for games & tags in the https://github.com/disconsented/workshop-walker/discussions
section.

## Contributing

Want to contribute a feature? Great!

Please suggest it as an _issue_ to get a feeling on whether I'll accept it or not, remember, that every piece of code
contributed adds to the maintenance burden. This is also an effort to manage burnout.

## APIReferences

https://partner.steamgames.com/doc/webapi/ipublishedfileservice

https://steamapi.xpaw.me/#IPublishedFileService/GetDetails

https://steamwebapi.azurewebsites.net/