use crate::rule_prelude::*;
use rslint_parser::TextRange;
use SyntaxKind::*;

declare_lint! {
    /**
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
    */
    #[serde(default)]
    NoIrregularWhitespace,
    errors,
    tags(Recommended),
    "no-irregular-whitespace",
    /// Whether to allow any whitespace in string literals (true by default)
    pub skip_strings: bool,
    /// Whether to allow any whitespace in comments (false by default)
    pub skip_comments: bool,
    /// Whether to allow any whitespace in regular expressions (false by default)
    pub skip_regex: bool,
    /// Whether to allow any whitespace in template literals (false by default)
    pub skip_templates: bool
}

impl Default for NoIrregularWhitespace {
    fn default() -> Self {
        Self {
            skip_strings: true,
            skip_comments: false,
            skip_regex: false,
            skip_templates: false,
        }
    }
}

const WHITESPACE_TABLE: [(char, &str); 24] = [
    ('\u{000B}', "Line Tabulation (\\v)"),
    ('\u{000C}', "Form Feed (\\f)"),
    ('\u{00A0}', "No-Break Space"),
    ('\u{0085}', "Next Line"),
    ('\u{1680}', "Ogham Space Mark"),
    ('\u{180E}', "Mongolian Vowel Separator"),
    ('\u{feff}', "Zero Width No-Break Space"),
    ('\u{2000}', "En Quad"),
    ('\u{2001}', "Em Quad"),
    ('\u{2002}', "En Space"),
    ('\u{2003}', "Em Space"),
    ('\u{2004}', "Tree-Per-Em"),
    ('\u{2005}', "Four-Per-Em"),
    ('\u{2006}', "Six-Per-Em"),
    ('\u{2007}', "Figure Space"),
    ('\u{2008}', "Punctuation Space"),
    ('\u{2009}', "Thin Space"),
    ('\u{200A}', "Hair Space"),
    ('\u{200B}', "Zero Width Space"),
    ('\u{2028}', "Line Separator"),
    ('\u{2029}', "Paragraph Separator"),
    ('\u{202F}', "Narrow No-Break space"),
    ('\u{205f}', "Medium Mathematical Space"),
    ('\u{3000}', "Ideographic Space"),
];

const FIRST_BYTES: [u8; 9] = [0x0b, 0x0c, 0xA0, 0x85, 0xC2, 0xE1, 0xEF, 0xE2, 0xE3];

// violations of this rule are extraordinarily rare, so we first run an initial pass which compares the first
// utf8 byte of each irregular whitespace with each byte in the string. This is extremely fast, since LLVM will
// turn it into a lookup table which is 3 operations to check each byte, and for x86 avx2 we use SIMD intrinsics.

// fallback for the short circuit pass
fn short_circuit_pass_fallback(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| FIRST_BYTES.contains(b))
}

// very fast short circuit pass using simd
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[target_feature(enable = "avx2")]
unsafe fn short_circuit_pass(bytes: &[u8]) -> bool {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::ptr::copy_nonoverlapping;

    // buffer for chunks which are not 32 items, we need this because if there is a chunk
    // that isnt 32 elements we cannot load a simd vector directly from it, because
    // that is undefined behavior (it is reading out of bounds memory)
    let mut buf = [0; 32];
    for chunk in bytes.chunks(32) {
        // if the chunk is 32 bits in length we can simply load a simd vector
        // using from its pointer
        let data = if chunk.len() == 32 {
            _mm256_loadu_si256(chunk.as_ptr() as *const __m256i)
        } else {
            // if the chunk isnt 32 bits in length then we need to copy the data
            // to the buffer or else that is undefined behavior
            copy_nonoverlapping(chunk.as_ptr(), buf.as_mut_ptr(), chunk.len());
            _mm256_loadu_si256(buf.as_ptr() as *const __m256i)
        };

        for byte in FIRST_BYTES.iter() {
            // This makes a 32 byte vector all consisting of the same byte
            let mask = _mm256_set1_epi8(*byte as i8);
            // Then we compare the data vector with the mask, if any byte corresponds to the checked
            // byte then it will become a `1` in the vector
            let cmp = _mm256_cmpeq_epi8(data, mask);
            // This will return `0` if there is no `1` in the vector
            if _mm256_movemask_epi8(cmp) != 0 {
                return true;
            }
        }
    }
    false
}

// slower pass which checks references to bytes, we can then convert matched references
// into a range by just comparing its adress against the first byte adress
#[inline]
fn spanned_byte_matches(bytes: &[u8]) -> Vec<usize> {
    let offset = bytes.as_ptr() as usize;

    bytes
        .iter()
        .filter(|byte| FIRST_BYTES.contains(byte))
        .map(|byte| byte as *const _ as usize - offset)
        .collect()
}

#[typetag::serde]
impl CstRule for NoIrregularWhitespace {
    fn check_root(&self, root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let string = root.text().to_string();
        let bytes = string.as_bytes();

        if string.is_empty() {
            return None;
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let res = if std::is_x86_feature_detected!("avx2") {
            // SAFETY: Explanation of the implementation and edge cases is in the function body
            unsafe { short_circuit_pass(bytes) }
        } else {
            short_circuit_pass_fallback(bytes)
        };

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let res = short_circuit_pass_fallback(bytes);

        if !res {
            return None;
        }

        // slow but still pretty fast path, we can get the byte ranges of offending bytes by just checking
        // the adress of the reference of each byte and subtracting the string pointer adress from it
        let matches = spanned_byte_matches(bytes);

        for byte_match in matches {
            // the byte may also be inside of a boundary, in which case, indexing into it is invalid so we need to handle this case
            if let Some(mut chars) = string.get(byte_match..).map(|x| x.chars()) {
                let offending_char = chars
                    .next()
                    .expect("Chars is an empty iterator even after a spanned byte match");
                // E2 and E3 obviously cover chars which are not offending chars, therefore we need to check if the char is actually right.
                let name = WHITESPACE_TABLE
                    .iter()
                    .find(|(c, _)| *c == offending_char)?
                    .1;

                self.maybe_throw_err(byte_match, name, offending_char, root, ctx);
            }
        }
        None
    }
}

impl NoIrregularWhitespace {
    fn maybe_throw_err(
        &self,
        byte_match: usize,
        name: &str,
        offending_char: char,
        root: &SyntaxNode,
        ctx: &mut RuleCtx,
    ) {
        let range = TextRange::new(
            (byte_match as u32).into(),
            ((offending_char.len_utf8() + byte_match) as u32).into(),
        );

        let cover = root.covering_element(range).into_token();

        if let Some(tok) = cover {
            match tok.kind() {
                COMMENT if self.skip_comments => return,
                REGEX if self.skip_regex => return,
                STRING if self.skip_strings => return,
                TEMPLATE_CHUNK if self.skip_templates => return,
                _ => {}
            }
        }

        let err = ctx
            .err(
                self.name(),
                format!("{} is not allowed to be used as whitespace", name),
            )
            .primary(
                range,
                format!("this character is a {}", name.to_ascii_lowercase()),
            );

        ctx.add_err(err);
    }
}

rule_tests! {
    NoIrregularWhitespace::default(),
    err: {
        "var any \u{000B} = 'thing';",
        "var any \u{000C} = 'thing';",
        "var any \u{00A0} = 'thing';",
        "var any \u{feff} = 'thing';",
        "var any \u{2000} = 'thing';",
        "var any \u{2001} = 'thing';",
        "var any \u{2002} = 'thing';",
        "var any \u{2003} = 'thing';",
        "var any \u{2004} = 'thing';",
        "var any \u{2005} = 'thing';",
        "var any \u{2006} = 'thing';",
        "var any \u{2007} = 'thing';",
        "var any \u{2008} = 'thing';",
        "var any \u{2009} = 'thing';",
        "var any \u{200A} = 'thing';",
        "var any \u{2028} = 'thing';",
        "var any \u{2029} = 'thing';",
        "var any \u{202F} = 'thing';",
        "var any \u{205f} = 'thing';",
        "var any \u{3000} = 'thing';"
    },
    ok: {
        "'\\u{000B}';",
        "'\\u{000C}';",
        "'\\u{0085}';",
        "'\\u{00A0}';",
        "'\\u{180E}';",
        "'\\u{feff}';",
        "'\\u{2000}';",
        "'\\u{2001}';",
        "'\\u{2002}';",
        "'\\u{2003}';",
        "'\\u{2004}';",
        "'\\u{2005}';",
        "'\\u{2006}';",
        "'\\u{2007}';",
        "'\\u{2008}';",
        "'\\u{2009}';",
        "'\\u{200A}';",
        "'\\u{200B}';",
        "'\\u{2028}';",
        "'\\u{2029}';",
        "'\\u{202F}';",
        "'\\u{205f}';",
        "'\\u{3000}';",
        "'\u{000B}';",
        "'\u{000C}';",
        "'\u{0085}';",
        "'\u{00A0}';",
        "'\u{180E}';",
        "'\u{feff}';",
        "'\u{2000}';",
        "'\u{2001}';",
        "'\u{2002}';",
        "'\u{2003}';",
        "'\u{2004}';",
        "'\u{2005}';",
        "'\u{2006}';",
        "'\u{2007}';",
        "'\u{2008}';",
        "'\u{2009}';",
        "'\u{200A}';",
        "'\u{200B}';",
        "'\\\u{2028}';",
        "'\\\u{2029}';",
        "'\u{202F}';",
        "'\u{205f}';",
        "'\u{3000}';"
    }
}
