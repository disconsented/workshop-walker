[![animation](https://raw.githubusercontent.com/orhun/git-cliff/main/website/static/img/git-cliff-anim.gif)](https://git-cliff.org)

## [unreleased]

### ⛰️  Features

- *(UI)* Multi-language support - ([a8abfe3](https://github.com/orhun/git-cliff/commit/a8abfe3a55c21fb04355acdeeb0cd2f210a36408))
- Small relations improvements - ([ee085c5](https://github.com/orhun/git-cliff/commit/ee085c5a8ab727b1d34c62d8abed757a4f8c7ff5))
- OpenGraph support - ([cb39050](https://github.com/orhun/git-cliff/commit/cb390504cb966d96f7f5230597502275ee00a6fc))
- "Suggest a game" on the index page. - ([59a8f9b](https://github.com/orhun/git-cliff/commit/59a8f9b958960d7b54d971992965756fd4f998db))
- Overhaul the items page - ([fe0ff7d](https://github.com/orhun/git-cliff/commit/fe0ff7dff8c1ba1ec50f7daa82830e32ed7544f8))
- Expose more item details - ([37d352c](https://github.com/orhun/git-cliff/commit/37d352c037489fc593e034b71dd5f72557f14bda))
- Sort by dependents - ([60dd036](https://github.com/orhun/git-cliff/commit/60dd03633753726e4056377dfed697000e9486c6))
- Add Spanish & Portuguese to supported languages - ([9390bae](https://github.com/orhun/git-cliff/commit/9390bae57b0ea6a9e11a489cc72415b289b3b1fe))
- Database migrations - ([dda1dc0](https://github.com/orhun/git-cliff/commit/dda1dc0a68c21284a3344db4dbaf1938853f277e))
- Better feature extraction - ([58618dd](https://github.com/orhun/git-cliff/commit/58618dd49629b7d7bed407f02d1eec4bc8d268d9))
- Display error messages for the item page - ([0200f71](https://github.com/orhun/git-cliff/commit/0200f710c3cd2de38bc5d2c248e16514a451aeba))
- Display error messages for the search page - ([72d7a65](https://github.com/orhun/git-cliff/commit/72d7a657dd8277c498ed1acc807707700594e2c5))
- Double pagination controls - ([42629e9](https://github.com/orhun/git-cliff/commit/42629e9c0b7efd813b47621181e97ab3b0395a7e))
- Page titles - ([1aa4b38](https://github.com/orhun/git-cliff/commit/1aa4b385786d66430481763f9acd4797f7e56a5f))
- Multi-language support - ([14d4b6f](https://github.com/orhun/git-cliff/commit/14d4b6fc2696afb759c233d3119a01561c2d6413))
- Adding contributing, suggestion sections & live link to readme.md - ([1066b45](https://github.com/orhun/git-cliff/commit/1066b45fec115fb14789793534dc9fa9d0cfdc55))
- Basic readme.md - ([24134e9](https://github.com/orhun/git-cliff/commit/24134e9e6410cf4ade408f6b2de7d52694bbc463))
- Add license - ([fe73c7b](https://github.com/orhun/git-cliff/commit/fe73c7bdbeff41b9e546bfc74e00ba202db9ef03))
- Add timeago to the item page - ([83406f4](https://github.com/orhun/git-cliff/commit/83406f47c4178fdb5418bcb07606f14388e60071))
- Add title/lastUpdated to stores - ([50fbd5a](https://github.com/orhun/git-cliff/commit/50fbd5a37ea378eb0db311331d67ec1670877fe1))
- Expose lastUpdated - ([1eb1a21](https://github.com/orhun/git-cliff/commit/1eb1a21b92ad5fc7688af7a0af020a2df8203bee))
- Add social links - ([b19205a](https://github.com/orhun/git-cliff/commit/b19205abd2fb3de2ee0ce2a6628a513ca311cca5))
- LLM experiments - ([8135f7a](https://github.com/orhun/git-cliff/commit/8135f7a5cf7d0814d701b96ac267eddcf8dcc1ad))
- Loading spinner - ([547227b](https://github.com/orhun/git-cliff/commit/547227bef1af5e01ac3a7007f1e2e3e86a3e6c0a))
- Add sorting by score - ([05952ea](https://github.com/orhun/git-cliff/commit/05952ea2049c16d8cf85f5adfd170756d10941f8))
- Title search - ([4240b79](https://github.com/orhun/git-cliff/commit/4240b793d18e862969c274b0138dc14cbc02afe8))
- Serve static files (ui) - ([4b562a2](https://github.com/orhun/git-cliff/commit/4b562a21869f31173f85edfae6d67f63c51f911d))
- Add periodic database updated - ([74990f6](https://github.com/orhun/git-cliff/commit/74990f6d5372bb7149c3f2bed651dac064fa1183))
- Add timeAgo.svelte component - ([f0d3647](https://github.com/orhun/git-cliff/commit/f0d3647c29a80b23238b86159e787543be45a0d0))
- Add support for language, order_by and limit to the backend requests - ([20e31ea](https://github.com/orhun/git-cliff/commit/20e31eae11617ebc37e7c9eac787fc96f7fb21ab))
- Display dep's in two rows to handle long titles better - ([c7d2668](https://github.com/orhun/git-cliff/commit/c7d266826fe1d9daeeaddef0f6927a6c478040da))
- List human time for updated instead of the timestamp - ([eaa34c6](https://github.com/orhun/git-cliff/commit/eaa34c6cc5193ed181dba100c9ea1ca2c9fd8927))
- Unique relationship keys for items - ([fa37406](https://github.com/orhun/git-cliff/commit/fa374066410b940d0a8b603e7cbfeed19447dbea))
- Expand language support - ([4498011](https://github.com/orhun/git-cliff/commit/4498011a46f2f6058949f8de07b2ad6802754ca4))

### 🐛 Bug Fixes

- "The field 'in' already exists" being reported by surrealdb - ([2f56cb1](https://github.com/orhun/git-cliff/commit/2f56cb1a2e2206b8538b571d75c38ed16c42c9fb))
- Surrealdb-migrations taking a minute to run - ([05d34d2](https://github.com/orhun/git-cliff/commit/05d34d2db0e11b4de4b74ba71dcbcda92ae85911))
- Resolve missing `children` field in the steam updater throwing errors. - ([1a97423](https://github.com/orhun/git-cliff/commit/1a97423875fe3a33fc2a8a36ef837feb0e61c884))
- Bind limit - ([24341e7](https://github.com/orhun/git-cliff/commit/24341e754797c98aff38209b700c77a768274fed))
- Init TimeAgo once - ([5edd287](https://github.com/orhun/git-cliff/commit/5edd2875d8913c330652e20c9e42569149675b86))
- Language query checking the wrong thing - ([c373895](https://github.com/orhun/git-cliff/commit/c3738957bdf18e3278d8469e4f403fc11884065f))
- Correct last_updated name - ([4004b64](https://github.com/orhun/git-cliff/commit/4004b64833175dfe14333ae743a8e11df74cc555))
- Always return unknown languages - ([d74c9b0](https://github.com/orhun/git-cliff/commit/d74c9b0c891a75ca241a72411e9501fa6b01cd22))
- Empty dir check returning the wrong count - ([60dd2d6](https://github.com/orhun/git-cliff/commit/60dd2d6d896b48d3d7f1afff301e824d8eff4f77))
- Database migration not raising errors - ([fd33876](https://github.com/orhun/git-cliff/commit/fd338761702324382ccf90ba4610a3aa820b6553))
- Loading data from steam not including last_updated - ([c9134d0](https://github.com/orhun/git-cliff/commit/c9134d0f64fcb689bc1c5d0f22edbd6005fa8287))
- Missing score in dep's lookup for item - ([baf44c4](https://github.com/orhun/git-cliff/commit/baf44c42843743726ef8a83edd19fbac93b70c1c))
- New database detection for docker - ([03bafc1](https://github.com/orhun/git-cliff/commit/03bafc1c9a7033081dc9048479e5878b26fa4879))
- Database creation - ([0ed35ff](https://github.com/orhun/git-cliff/commit/0ed35ff99313131e8a3337198073e8eceec15fb6))
- Missing table schema for "workshop_items" - ([de048eb](https://github.com/orhun/git-cliff/commit/de048eb3a27b95399223c7ee8e4f1e3e46067361))
- Early exit rather than stall when getting blank response from steam - ([7b73592](https://github.com/orhun/git-cliff/commit/7b735929d66b92cf2ba2078bb498b68a2e9fadb4))

### ⚡ Performance

- Target x86-64-v3 for x86_64 builds - ([e676304](https://github.com/orhun/git-cliff/commit/e676304ee6667d5e5cb9970406ccedaae128ced9))
- Use language ID's instead of language names for querying - ([4b7e741](https://github.com/orhun/git-cliff/commit/4b7e74103967db6841691c6c18aeec43167928e1))
- Speed up list queries from about 550ms to about 2ms - ([09599ad](https://github.com/orhun/git-cliff/commit/09599ad8df9ea5d162553d5e9d1a6279f6b808ce))
- Use built in to_string function for id conversion - ([22b2d7f](https://github.com/orhun/git-cliff/commit/22b2d7f06055228ba032e4b13113ec2118ebef1d))
- Database optimisations - ([ab698f5](https://github.com/orhun/git-cliff/commit/ab698f51c4c9dd81cabd514cce36df8d7e49a850))
- Add database indexes - ([47a610f](https://github.com/orhun/git-cliff/commit/47a610f81c88769e110188b6c19a88ad0bc1795e))

### 🎨 Styling

- Convert tag's to use skeleton badges - ([324c1ab](https://github.com/orhun/git-cliff/commit/324c1abb5649b550648dbce14d8d1a223c42c572))

### ⚙️ Miscellaneous Tasks

- Update Cargo.lock - ([35b435c](https://github.com/orhun/git-cliff/commit/35b435cfe9d1b54a00de7a690590a7a28b322859))
- Update .gitignore - ([0e9f942](https://github.com/orhun/git-cliff/commit/0e9f9420fe8ece651606d6ea06a3cb502b87c65f))
- Swap over to surrealdb-migrations fork - ([f148b1d](https://github.com/orhun/git-cliff/commit/f148b1d9cb5e7f946daee9a497161d0c71f61cb0))
- Swap over to surrealdb-migrations fork - ([20a37e4](https://github.com/orhun/git-cliff/commit/20a37e43a2941fc8358cb1cb5d32963a1330064a))
- Item_dependencies.surql formatting - ([cff76fd](https://github.com/orhun/git-cliff/commit/cff76fd180ff817814cd1d981e23759d728d9dc0))
- Migration logging - ([d9d50ac](https://github.com/orhun/git-cliff/commit/d9d50ac7b38537ee47b4372b6df3bdb1a37db5d2))
- Cargo update - ([4a9bf9f](https://github.com/orhun/git-cliff/commit/4a9bf9fd7b1f195d72eade42add6afeb1984475f))
- Misc fixes and formatting - ([7e75bc2](https://github.com/orhun/git-cliff/commit/7e75bc28eae7daa9f12dcce80919e93b208a4d5c))
- Misc fixes and formatting - ([bb5a8ab](https://github.com/orhun/git-cliff/commit/bb5a8abbe5c0459499c1cc91b131b9919587bfee))
- Tidy up deleted files - ([0ac4498](https://github.com/orhun/git-cliff/commit/0ac4498a37eb0fd08b5e443c890d9b4adbcd9c91))
- .gitignore updates - ([33db809](https://github.com/orhun/git-cliff/commit/33db809503fa60d21178b2a36f17daa64100eca7))
- Lints - ([0633c3c](https://github.com/orhun/git-cliff/commit/0633c3c8241f18526837beb620ac3b799bf2f6f0))
- Format - ([6bd813e](https://github.com/orhun/git-cliff/commit/6bd813e7ab82016b0341bd5f30fa7acf600adf52))
- Add missing files for LLM experiments - ([a95b96e](https://github.com/orhun/git-cliff/commit/a95b96e1109eeff940c8f9746759d3d3196a1f4b))
- Covert labels to spans - ([2118d19](https://github.com/orhun/git-cliff/commit/2118d1980b27f7c90cdb6af65344a5e10b5ec807))
- Tidying - ([7f23da7](https://github.com/orhun/git-cliff/commit/7f23da71b5100963d075f58537f9a84498714d0f))
- Change over to more sensible link targets - ([eaa6be8](https://github.com/orhun/git-cliff/commit/eaa6be830ee313c40aa9b48a2ae7ce467281d494))
- Sync dependencies - ([b6ec12b](https://github.com/orhun/git-cliff/commit/b6ec12bd1bfa991418b13140c05a808991b6a5ee))
- Remove docker entry in dockerfile - ([da44457](https://github.com/orhun/git-cliff/commit/da44457997636a410bdbfdef5b3a838b0d4d4d63))
- Lints and tweaks - ([3c2ebf7](https://github.com/orhun/git-cliff/commit/3c2ebf710ba36336a87fb4460c9da6ae04af031d))
- Change location for the config file in .gitignore - ([b7acbda](https://github.com/orhun/git-cliff/commit/b7acbda76113a948ef06479c446cae73c8687923))
- Formatting - ([30ca466](https://github.com/orhun/git-cliff/commit/30ca466b9d2e6cf1680c0c13030269701dba32b2))
- Lazy push - ([e4c59d2](https://github.com/orhun/git-cliff/commit/e4c59d2000d339da16bc455922793d2ccc2006f2))

### Build

- Add .dockerignore - ([d1d3599](https://github.com/orhun/git-cliff/commit/d1d35995ffdcc3e7e6ae18e7705b4cc8efedd96f))
- Fix build issues - ([7a2572f](https://github.com/orhun/git-cliff/commit/7a2572f90cd64698a2180266f399aacca006d4cd))
- Add mold linker - ([0e77406](https://github.com/orhun/git-cliff/commit/0e77406531d33a73dca2d1587c8c69dc0ab04783))
- Swap to a different build container for the UI, fixing the ui build - ([8511fcf](https://github.com/orhun/git-cliff/commit/8511fcfe4d3060d2c833d38774ca5538c5a4a8a6))
- Static rendering for the UI - ([fbbe321](https://github.com/orhun/git-cliff/commit/fbbe321aacb46c2f83dd1b3f524c444a049d7a9c))
- Docker support - ([e59c6a2](https://github.com/orhun/git-cliff/commit/e59c6a29c3fe079d891f3e8a571586e74856180b))

### Fear

- Initial UI - ([d248e72](https://github.com/orhun/git-cliff/commit/d248e72f202b887bf5130d1832f9f38f46a773a1))


<!-- generated by git-cliff -->
