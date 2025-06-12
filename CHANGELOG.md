# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/disconsented/workshop-walker/releases/tag/workshop-walker-v0.1.0) - 2025-06-12

### Added

- added multiline description + nice gradient
- Add quick selection options for updated since
- Add tags back to the item page
- Small relations improvements
- OpenGraph support
- "Suggest a game" on the index page.
- Overhaul the items page
- Expose more item details
- Sort by dependents
- Add Spanish & Portuguese to supported languages
- Database migrations
- Better feature extraction
- Display error messages for the item page
- Display error messages for the search page
- Double pagination controls
- Page titles
- *(UI)* Multi-language support
- Multi-language support
- Adding contributing, suggestion sections & live link to readme.md
- Basic readme.md
- Add license
- Add timeago to the item page
- Add title/lastUpdated to stores
- Expose lastUpdated
- Add social links
- LLM experiments
- Loading spinner
- Add sorting by score
- Title search
- Serve static files (ui)
- Add periodic database updated
- Add timeAgo.svelte component
- Add support for language, order_by and limit to the backend requests
- Display dep's in two rows to handle long titles better
- List human time for updated instead of the timestamp
- Unique relationship keys for items
- Expand language support

### Fixed

- Grid view overflowing the container
- False positive language detection
- "The field 'in' already exists" being reported by surrealdb
- Surrealdb-migrations taking a minute to run
- Resolve missing `children` field in the steam updater throwing errors.
- Bind limit
- Init TimeAgo once
- Language query checking the wrong thing
- Correct last_updated name
- Always return unknown languages
- Empty dir check returning the wrong count
- Database migration not raising errors
- Loading data from steam not including last_updated
- Missing score in dep's lookup for item
- New database detection for docker
- Database creation
- Missing table schema for "workshop_items"
- Early exit rather than stall when getting blank response from steam

### Other

- Correct branch name for release.yml
- Add paraglide files
- *(CI)* Add release workflow
- *(CI)* Add lint workflow
- Add the 1.6 tag to the app page
- Add paraglide files
- Formatting and misc changes
- Update default limit for app search to 50
- Increase default pagination size for the app view
- App search is now responsive
- Add changelog.md
- Add git-cliff support
- Correct surrealdb-migrations source
- Add missing migrations files
- Update Cargo.lock
- Update .gitignore
- Swap over to surrealdb-migrations fork
- Swap over to surrealdb-migrations fork
- item_dependencies.surql formatting
- Migration logging
- Cargo update
- Misc fixes and formatting
- Misc fixes and formatting
- Add .dockerignore
- Tidy up deleted files
- .gitignore updates
- Target x86-64-v3 for x86_64 builds
- Fix build issues
- Lints
- Format
- Add missing files for LLM experiments
- Covert labels to spans
- tidying
- Change over to more sensible link targets
- Convert tag's to use skeleton badges
- sync dependencies
- Use language ID's instead of language names for querying
- Remove docker entry in dockerfile
- Add mold linker
- Speed up list queries from about 550ms to about 2ms
- Use built in to_string function for id conversion
- Swap to a different build container for the UI, fixing the ui build
- Lints and tweaks
- Database optimisations
- Change location for the config file in .gitignore
- Static rendering for the UI
- Formatting
- Docker support
- Lazy push
- Add database indexes
- Initial UI
- initial commit
