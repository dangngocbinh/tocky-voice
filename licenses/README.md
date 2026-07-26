# Third-party licences

Tocky Voice itself is [MIT](../LICENSE). This directory holds the licences of things
bundled *inside* the application, which have their own terms.

## Fonts

Both are shipped as `.woff2` files inside the app, so their licences travel with it. The
SIL Open Font License requires the copyright notice and licence text to accompany any
redistribution of the font — that is what these files are for.

| Font | Copyright | Licence |
| --- | --- | --- |
| Be Vietnam Pro | Copyright 2021 The Be Vietnam Pro Project Authors ([bettergui/BeVietnamPro](https://github.com/bettergui/BeVietnamPro)) | [OFL 1.1](OFL-Be-Vietnam-Pro.txt) |
| JetBrains Mono | Copyright 2020 The JetBrains Mono Project Authors ([JetBrains/JetBrainsMono](https://github.com/JetBrains/JetBrainsMono)) | [OFL 1.1](OFL-JetBrains-Mono.txt) |

Neither font is modified. Under OFL 1.1 they may be bundled and redistributed with
software, including commercially; what is not allowed is selling the fonts on their own,
or releasing a modified version under the original Reserved Font Name.

## Code dependencies

Audited across 651 Rust crates and the four production npm packages: everything is
MIT, Apache-2.0, BSD, Zlib, Unicode-3.0, MPL-2.0, or a choice that includes one of those.
**No GPL, LGPL-only, AGPL or SSPL code is linked**, so distributing a binary under MIT
carries no copyleft obligation.

Five crates are MPL-2.0 (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`,
`selectors`), pulled in transitively by Tauri. MPL-2.0 is file-level copyleft: it applies
to those files, not to the program that links them, and only if they are modified. They
are not modified here.

To re-check after a dependency change:

```sh
cd src-tauri && cargo metadata --format-version 1 | \
  python3 -c "import json,sys; [print(p['name'], p.get('license')) for p in json.load(sys.stdin)['packages'] if 'GPL' in (p.get('license') or '')]"
```
