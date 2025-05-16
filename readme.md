## Workshop Walker

Workshop Walker is a "better" interface to the steam workshop, aiming to solve some of the limitations that I personally
encounter whilst browsing Rimworld mods.

- Language filtering support
- Discovering _dependants_ for a mod
- Better classification for existing mods

## How it works

Workshop Walker (WW) makes
use of [SurrealDB's relationship modeling](https://surrealdb.com/docs/surrealql/datamodel/ids), where the dependency
relationships can be efficiently reversed. Which gives us cheap dependant lookups.

Language support is handled heuristically by https://crates.io/crates/lingua

Everything else is pretty straight forward.

## APIReferences

https://partner.steamgames.com/doc/webapi/ipublishedfileservice

https://steamapi.xpaw.me/#IPublishedFileService/GetDetails

https://steamwebapi.azurewebsites.net/