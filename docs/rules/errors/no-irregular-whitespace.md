<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-irregular-whitespace

Disallow weird/irregular whitespace.

ECMAScript allows a wide selection of unicode whitespace, they are however known to
cause issues with various parsers, therefore they should never be used.

A lot of the whitespace is invisible, therefore is hard to detect, it may have been inserted
by accident.

Whitespace such as line separator causes issues since line separators are not valid JSON which
may cause many issues.

This rule disallows the following whitespace:

```text
\u000B - Line Tabulation (\v) - <VT>
\u000C - Form Feed (\f) - <FF>
\u00A0 - No-Break Space - <NBSP>
\u0085 - Next Line
\u1680 - Ogham Space Mark
\u180E - Mongolian Vowel Separator - <MVS>
\ufeff - Zero Width No-Break Space - <BOM>
\u2000 - En Quad
\u2001 - Em Quad
\u2002 - En Space - <ENSP>
\u2003 - Em Space - <EMSP>
\u2004 - Tree-Per-Em
\u2005 - Four-Per-Em
\u2006 - Six-Per-Em
\u2007 - Figure Space
\u2008 - Punctuation Space - <PUNCSP>
\u2009 - Thin Space
\u200A - Hair Space
\u200B - Zero Width Space - <ZWSP>
\u2028 - Line Separator
\u2029 - Paragraph Separator
\u202F - Narrow No-Break Space
\u205f - Medium Mathematical Space
\u3000 - Ideographic Space
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `skipStrings` | bool |  Whether to allow any whitespace in string literals (true by default) |
| `skipComments` | bool |  Whether to allow any whitespace in comments (false by default) |
| `skipRegex` | bool |  Whether to allow any whitespace in regular expressions (false by default) |
| `skipTemplates` | bool |  Whether to allow any whitespace in template literals (false by default) |

<details>
 <summary> More incorrect examples </summary>

```js
var any  = 'thing';
```

```js
var any  = 'thing';
```

```js
var any   = 'thing';
```

```js
var any ﻿ = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any   = 'thing';
```

```js
var any 　 = 'thing';
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
'\u{000B}';
```

```js
'\u{000C}';
```

```js
'\u{0085}';
```

```js
'\u{00A0}';
```

```js
'\u{180E}';
```

```js
'\u{feff}';
```

```js
'\u{2000}';
```

```js
'\u{2001}';
```

```js
'\u{2002}';
```

```js
'\u{2003}';
```

```js
'\u{2004}';
```

```js
'\u{2005}';
```

```js
'\u{2006}';
```

```js
'\u{2007}';
```

```js
'\u{2008}';
```

```js
'\u{2009}';
```

```js
'\u{200A}';
```

```js
'\u{200B}';
```

```js
'\u{2028}';
```

```js
'\u{2029}';
```

```js
'\u{202F}';
```

```js
'\u{205f}';
```

```js
'\u{3000}';
```

```js
'';
```

```js
'';
```

```js
'';
```

```js
' ';
```

```js
'᠎';
```

```js
'﻿';
```

```js
' ';
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_irregular_whitespace.rs)