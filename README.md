# goglob

Shell pattern matching similar to golang's `path.Match`.

The pattern syntax is:

```
pattern:
    { term }
term:
    '*'         matches any sequence of non-/ characters
    '?'         matches any single non-/ character
    '[' [ '^' ] { character-range } ']'
                character class (must be non-empty)
    c           matches character c (c != '*', '?', '\\', '[')
    '\\' c      matches character c

character-range:
    c           matches character c (c != '\\', '-', ']')
    '\\' c      matches character c
    lo '-' hi   matches character c for lo <= c <= hi
```

Match requires pattern to match all of name, not just a substring.

Use `GlobPattern::new(pattern)` to construct a new instance.

## Features
* `proc-macro`: allows using the `glob!("<PATTERN>")` procedural macro (see
  `glob!()`).
* `serde`: enables serde deserialization of string patterns.

## License
`BSD-3-Clause`.

Based in [Go 1.18's `path.Match`](https://cs.opensource.google/go/go/+/refs/tags/go1.18:src/path/match.go;l=38)
(available under the [`BSD-3-Clause` license](https://cs.opensource.google/go/go/+/refs/tags/go1.18:LICENSE))
which contains the following comment:

```go
// Copyright 2010 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
```
