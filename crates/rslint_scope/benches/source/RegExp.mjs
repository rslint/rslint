import unicodeCaseFoldingCommon from 'unicode-13.0.0/Case_Folding/C/symbols.js';
import unicodeCaseFoldingSimple from 'unicode-13.0.0/Case_Folding/S/symbols.js';
import { surroundingAgent } from '../engine.mjs';
import { Type, Value } from '../value.mjs';
import { Assert, IsNonNegativeInteger } from '../abstract-ops/all.mjs';
import { CharacterValue, StringToCodePoints } from '../static-semantics/all.mjs';
import { X } from '../completion.mjs';
import { isLineTerminator, isWhitespace, isDecimalDigit } from '../parser/Lexer.mjs';
import { OutOfRange } from '../helpers.mjs';
import {
    UnicodeMatchProperty,
    UnicodeMatchPropertyValue,
    UnicodeGeneralCategoryValues,
    BinaryUnicodeProperties,
    NonbinaryUnicodeProperties,
    getUnicodePropertyValueSet,
} from './all.mjs';

// #sec-pattern
class State {
    constructor(endIndex, captures) {
        this.endIndex = endIndex;
        this.captures = captures;
    }
}

export { State as RegExpState };

function isContinuation(v) {
    return typeof v === 'function' && v.length === 1;
}

class CharSet {
    union(other) {
        const concrete = new Set();
        const fns = new Set();
        const add = (cs) => {
            if (cs.fns) {
                cs.fns.forEach((fn) => {
                    fns.add(fn);
                });
                cs.concrete.forEach((c) => {
                    concrete.add(c);
                });
            } else if (cs.fn) {
                fns.add(cs.fn);
            } else {
                cs.concrete.forEach((c) => {
                    concrete.add(c);
                });
            }
        };
        add(this);
        add(other);
        return new UnionCharSet(concrete, fns);
    }
}

class UnionCharSet extends CharSet {
    constructor(concrete, fns) {
        super();

        this.concrete = concrete;
        this.fns = fns;
    }

    has(c) {
        if (this.concrete.has(c)) {
            return true;
        }
        for (const fn of this.fns) {
            if (fn(c)) {
                return true;
            }
        }
        return false;
    }
}

class ConcreteCharSet extends CharSet {
    constructor(items) {
        super();
        this.concrete = items instanceof Set ? items : new Set(items);
    }

    has(c) {
        return this.concrete.has(c);
    }

    get size() {
        return this.concrete.size;
    }

    first() {
        Assert(this.concrete.size >= 1);
        return this.concrete.values().next().value;
    }
}

class VirtualCharSet extends CharSet {
    constructor(fn) {
        super();
        this.fn = fn;
    }

    has(c) {
        return this.fn(c);
    }
}

class Range {
    constructor(startIndex, endIndex) {
        Assert(startIndex <= endIndex);
        this.startIndex = startIndex;
        this.endIndex = endIndex;
    }
}

// #sec-pattern
//   Pattern :: Disjunction
export function Evaluate_Pattern(Pattern, flags) {
    // The descriptions below use the following variables:
    //   * Input is a List consisting of all of the characters, in order, of the String being matched
    //     by the regular expression pattern. Each character is either a code unit or a code point,
    //     depending upon the kind of pattern involved. The notation Input[n] means the nth character
    //     of Input, where n can range between 0 (inclusive) and InputLength (exclusive).
    //   * InputLength is the number of characters in Input.
    //   * NcapturingParens is the total number of left-capturing parentheses (i.e. the total number of
    //     Atom :: `(` GroupSpecifier Disjunction `)` Parse Nodes) in the pattern. A left-capturing parenthesis
    //     is any `(` pattern character that is matched by the `(` terminal of the Atom :: `(` GroupSpecifier Disjunction `)`
    //     production.
    //   * DotAll is true if the RegExp object's [[OriginalFlags]] internal slot contains "s" and otherwise is false.
    //   * IgnoreCase is true if the RegExp object's [[OriginalFlags]] internal slot contains "i" and otherwise is false.
    //   * Multiline is true if the RegExp object's [[OriginalFlags]] internal slot contains "m" and otherwise is false.
    //   * Unicode is true if the RegExp object's [[OriginalFlags]] internal slot contains "u" and otherwise is false.
    let Input;
    let InputLength;
    const NcapturingParens = Pattern.capturingGroups.length;
    const DotAll = flags.includes('s');
    const IgnoreCase = flags.includes('i');
    const Multiline = flags.includes('m');
    const Unicode = flags.includes('u');

    {
        // 1. Evaluate Disjunction with +1 as its direction argument to obtain a Matcher m.
        const m = Evaluate(Pattern.Disjunction, +1);
        // 2. Return a new abstract closure with parameters (str, index) that captures m and performs the following steps when called:
        return (str, index) => {
            // a. Assert: Type(str) is String.
            Assert(Type(str) === 'String');
            // b. Assert: ! IsNonNegativeInteger(index) is true and index ≤ the length of str.
            Assert(X(IsNonNegativeInteger(index)) === Value.true
                && index.numberValue() <= str.stringValue().length);
            // c. If Unicode is true, let Input be a List consisting of the sequence of code points of ! StringToCodePoints(str).
            //    Otherwise, let Input be a List consisting of the sequence of code units that are the elements of str.
            //    Input will be used throughout the algorithms in 21.2.2. Each element of Input is considered to be a character.
            if (Unicode) {
                Input = X(StringToCodePoints(str.stringValue()));
            } else {
                Input = str.stringValue().split('').map((c) => c.charCodeAt(0));
            }
            // d. Let InputLength be the number of characters contained in Input. This variable will be used throughout the algorithms in 21.2.2.
            InputLength = Input.length;
            // e. Let listIndex be the index into Input of the character that was obtained from element index of str.
            const listIndex = index.numberValue();
            // f. Let c be a new Continuation with parameters (y) that captures nothing and performs the following steps when called:
            const c = (y) => {
                // i. Assert: y is a State.
                Assert(y instanceof State);
                // ii. Return y.
                return y;
            };
            // g. Let cap be a List of NcapturingParens undefined values, indexed 1 through NcapturingParens.
            const cap = Array.from({ length: NcapturingParens + 1 }, () => Value.undefined);
            // h. Let x be the State (listIndex, cap).
            const x = new State(listIndex, cap);
            // i. Call m(x, c) and return its result.
            return m(x, c);
        };
    }

    function Evaluate(node, ...args) {
        switch (node.type) {
            case 'Disjunction':
                return Evaluate_Disjunction(node, ...args);
            case 'Alternative':
                return Evaluate_Alternative(node, ...args);
            case 'Term':
                return Evaluate_Term(node, ...args);
            case 'Assertion':
                return Evaluate_Assertion(node, ...args);
            case 'Quantifier':
                return Evaluate_Quantifier(node, ...args);
            case 'Atom':
                return Evaluate_Atom(node, ...args);
            case 'AtomEscape':
                return Evaluate_AtomEscape(node, ...args);
            case 'CharacterEscape':
                return Evaluate_CharacterEscape(node, ...args);
            case 'DecimalEscape':
                return Evaluate_DecimalEscape(node, ...args);
            case 'CharacterClassEscape':
                return Evaluate_CharacterClassEscape(node, ...args);
            case 'UnicodePropertyValueExpression':
                return Evaluate_UnicodePropertyValueExpression(node, ...args);
            case 'CharacterClass':
                return Evaluate_CharacterClass(node, ...args);
            case 'ClassAtom':
                return Evaluate_ClassAtom(node, ...args);
            case 'ClassEscape':
                return Evaluate_ClassEscape(node, ...args);
            default:
                throw new OutOfRange('Evaluate', node);
        }
    }

    // #sec-disjunction
    //   Disjunction ::
    //     Alternative
    //     Alternative `|` Disjunction
    function Evaluate_Disjunction({ Alternative, Disjunction }, direction) {
        if (!Disjunction) {
            // 1. Evaluate Alternative with argument direction to obtain a Matcher m.
            const m = Evaluate(Alternative, direction);
            // 2. Return m.
            return m;
        }
        // 1. Evaluate Alternative with argument direction to obtain a Matcher m1.
        const m1 = Evaluate(Alternative, direction);
        // 2. Evaluate Disjunction with argument direction to obtain a Matcher m2.
        const m2 = Evaluate(Disjunction, direction);
        // 3. Return a new Matcher with parameters (x, c) that captures m1 and m2 and performs the following steps when called:
        return (x, c) => {
            // a. Assert: x is a State.
            Assert(x instanceof State);
            // b. Assert: c is a Continuation.
            Assert(isContinuation(c));
            // c. Call m1(x, c) and let r be its result.
            const r = m1(x, c);
            // d. If r is not failure, return r.
            if (r !== 'failure') {
                return r;
            }
            // e. Call m2(x, c) and return its result.
            return m2(x, c);
        };
    }

    // #sec-alternative
    //   Alternative ::
    //     [empty]
    //     Alternative Term
    function Evaluate_Alternative({ Alternative, Term }, direction) {
        if (!Alternative && !Term) {
            // 1. Return a new Matcher with parameters (x, c) that captures nothing and performs the following steps when called:
            return (x, c) => {
                // 1. Assert: x is a State.
                Assert(x instanceof State);
                // 2. Assert: c is a Continuation.
                Assert(isContinuation(c));
                // 3. Call c(x) and return its result.
                return c(x);
            };
        }
        // 1. Evaluate Alternative with argument direction to obtain a Matcher m1.
        const m1 = Evaluate(Alternative, direction);
        // 2. Evaluate Term with argument direction to obtain a Matcher m2.
        const m2 = Evaluate(Term, direction);
        // 3. If direction is equal to +1, then
        if (direction === +1) {
            // a. Return a new Matcher with parameters (x, c) that captures m1 and m2 and performs the following steps when called:
            return (x, c) => {
                // i. Assert: x is a State.
                Assert(x instanceof State);
                // ii. Assert: c is a Continuation.
                Assert(isContinuation(c));
                // iii. Let d be a new Continuation with parameters (y) that captures c and m2 and performs the following steps when called:
                const d = (y) => {
                    // 1. Assert: y is a State.
                    Assert(y instanceof State);
                    // 2. Call m2(y, c) and return its result.
                    return m2(y, c);
                };
                // iv. Call m1(x, d) and return its result.
                return m1(x, d);
            };
        } else { // 4. Else,
            // a. Assert: direction is equal to -1.
            Assert(direction === -1);
            // b. Return a new Matcher with parameters (x, c) that captures m1 and m2 and performs the following steps when called:
            return (x, c) => {
                // i. Assert: x is a State.
                Assert(x instanceof State);
                // ii. Assert: c is a Continuation.
                Assert(isContinuation(c));
                // iii. Let d be a new Continuation with parameters (y) that captures c and m1 and performs the following steps when called:
                const d = (y) => {
                    // 1. Assert: y is a State.
                    Assert(y instanceof State);
                    // 2. Call m1(y, c) and return its result.
                    return m1(y, c);
                };
                // iv. Call m2(x, d) and return its result.
                return m2(x, d);
            };
        }
    }

    // #sec-term
    //   Term ::
    //     Assertion
    //     Atom
    //     Atom Quantifier
    function Evaluate_Term(Term, direction) {
        const { Atom, Quantifier } = Term;
        if (!Quantifier) {
            // 1. Return the Matcher that is the result of evaluating Atom with argument direction.
            return Evaluate(Atom, direction);
        }
        // 1. Evaluate Atom with argument direction to obtain a Matcher m.
        const m = Evaluate(Atom, direction);
        // 2. Evaluate Quantifier to obtain the three results: an integer min, an integer (or ∞) max, and Boolean greedy.
        const [min, max, greedy] = Evaluate(Quantifier);
        // 3. Assert: If max is finite, then max is not less than min.
        Assert(!Number.isFinite(max) || (max >= min));
        // 4. Let parenIndex be the number of left-capturing parentheses in the entire regular expression that occur to the
        //    left of this Term. This is the total number of Atom :: `(` GroupSpecifier Disjunction `)` Parse Nodes prior to
        //    or enclosing this Term.
        const parenIndex = Term.capturingParenthesesBefore;
        // 5. Let parenCount be the number of left-capturing parentheses in Atom. This is the total number of
        //    Atom :: `(` GroupSpecifier Disjunction `)` Parse Nodes enclosed by Atom.
        const parenCount = Atom.enclosedCapturingParentheses;
        // 6. Return a new Matcher with parameters (x, c) that captures m, min, max, greedy, parenIndex, and parenCount and performs the following steps when called:
        return (x, c) => {
            // a. Assert: x is a State.
            Assert(x instanceof State);
            // b. Assert: c is a Continuation.
            Assert(isContinuation(c));
            // c. Call RepeatMatcher(m, min, max, greedy, x, c, parenIndex, parenCount) and return its result.
            return RepeatMatcher(m, min, max, greedy, x, c, parenIndex, parenCount);
        };
    }

    // #sec-runtime-semantics-repeatmatcher-abstract-operation
    function RepeatMatcher(m, min, max, greedy, x, c, parenIndex, parenCount) {
        // 1. If max is zero, return c(x).
        if (max === 0) {
            return c(x);
        }
        // 2. Let d be a new Continuation with parameters (y) that captures m, min, max, greedy, x, c, parenIndex, and parenCount and performs the following steps when called:
        const d = (y) => {
            // a. Assert: y is a State.
            Assert(y instanceof State);
            // b. If min is zero and y's endIndex is equal to x's endIndex, return failure.
            if (min === 0 && y.endIndex === x.endIndex) {
                return 'failure';
            }
            // c. If min is zero, let min2 be zero; otherwise let min2 be min - 1.
            let min2;
            if (min === 0) {
                min2 = 0;
            } else {
                min2 = min - 1;
            }
            // d. If max is ∞, let max2 be ∞; otherwise let max2 be max - 1.
            let max2;
            if (max === Infinity) {
                max2 = Infinity;
            } else {
                max2 = max - 1;
            }
            // e. Call RepeatMatcher(m, min2, max2, greedy, y, c, parenIndex, parenCount) and return its result.
            return RepeatMatcher(m, min2, max2, greedy, y, c, parenIndex, parenCount);
        };
        // 3. Let cap be a copy of x's captures List.
        const cap = [...x.captures];
        // 4. For each integer k that satisfies parenIndex < k and k ≤ parenIndex + parenCount, set cap[k] to undefined.
        for (let k = parenIndex + 1; k <= parenIndex + parenCount; k += 1) {
            cap[k] = Value.undefined;
        }
        // 5. Let e be x's endIndex.
        const e = x.endIndex;
        // 6. Let xr be the State (e, cap).
        const xr = new State(e, cap);
        // 7. If min is not zero, return m(xr, d).
        if (min !== 0) {
            return m(xr, d);
        }
        // 8. If greedy is false, then
        if (greedy === false) {
            // a. Call c(x) and let z be its result.
            const z = c(x);
            // b. If z is not failure, return z.
            if (z !== 'failure') {
                return z;
            }
            // c. Call m(xr, d) and return its result.
            return m(xr, d);
        }
        // 9. Call m(xr, d) and let z be its result.
        const z = m(xr, d);
        // 10. If z is not failure, return z.
        if (z !== 'failure') {
            return z;
        }
        // 11. Call c(x) and return its result.
        return c(x);
    }

    // #sec-assertion
    //   Assertion ::
    //     `^`
    //     `$`
    //     `\` `b`
    //     `\` `B`
    //     `(` `?` `=` Disjunction `)`
    //     `(` `?` `!` Disjunction `)`
    //     `(` `?` `<=` Disjunction `)`
    //     `(` `?` `<!` Disjunction `)`
    function Evaluate_Assertion({ subtype, Disjunction }) {
        switch (subtype) {
            case '^':
                // 1. Return a new Matcher with parameters (x, c) that captures nothing and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let e be x's endIndex.
                    const e = x.endIndex;
                    // d. If e is zero, or if Multiline is true and the character Input[e - 1] is one of LineTerminator, then
                    if (e === 0 || (Multiline && isLineTerminator(String.fromCodePoint(Input[e - 1])))) {
                        // i. Call c(x) and return its result.
                        return c(x);
                    }
                    // e. Return failure.
                    return 'failure';
                };
            case '$':
                // 1. Return a new Matcher with parameters (x, c) that captures nothing and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let e be x's endIndex.
                    const e = x.endIndex;
                    // d. If e is equal to InputLength, or if Multiline is true and the character Input[e] is one of LineTerminator, then
                    if (e === InputLength || (Multiline && isLineTerminator(String.fromCodePoint(Input[e])))) {
                        // i. Call c(x) and return its result.
                        return c(x);
                    }
                    // e. Return failure.
                    return 'failure';
                };
            case 'b':
                // 1. Return a new Matcher with parameters (x, c) that captures nothing and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let e be x's endIndex.
                    const e = x.endIndex;
                    // d. Call IsWordChar(e - 1) and let a be the Boolean result.
                    const a = IsWordChar(e - 1);
                    // e. Call IsWordChar(e) and let b be the Boolean result.
                    const b = IsWordChar(e);
                    // f. If a is true and b is false, or if a is false and b is true, then
                    if ((a && !b) || (!a && b)) {
                        // i. Call c(x) and return its result.
                        return c(x);
                    }
                    // g. Return failure.
                    return 'failure';
                };
            case 'B':
                // 1. Return a new Matcher with parameters (x, c) that captures nothing and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let e be x's endIndex.
                    const e = x.endIndex;
                    // d. Call IsWordChar(e - 1) and let a be the Boolean result.
                    const a = IsWordChar(e - 1);
                    // e. Call IsWordChar(e) and let b be the Boolean result.
                    const b = IsWordChar(e);
                    // f. If a is true and b is true, or if a is false and b is false, then
                    if ((a && b) || (!a && !b)) {
                        // i. Call c(x) and return its result.
                        return c(x);
                    }
                    // g. Return failure.
                    return 'failure';
                };
            case '?=': {
                // 1. Evaluate Disjunction with +1 as its direction argument to obtain a Matcher m.
                const m = Evaluate(Disjunction, +1);
                // 2. Return a new Matcher with parameters (x, c) that captures m and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let d be a new Continuation with parameters (y) that captures nothing and performs the following steps when called:
                    const d = (y) => {
                        // i. Assert: y is a State.
                        Assert(y instanceof State);
                        // ii. Return y.
                        return y;
                    };
                    // d. Call m(x, d) and let r be its result.
                    const r = m(x, d);
                    // e. If r is failure, return failure.
                    if (r === 'failure') {
                        return 'failure';
                    }
                    // f. Let y be r's State.
                    const y = r;
                    // g. Let cap be y's captures List.
                    const cap = y.captures;
                    // h. Let xe be x's endIndex.
                    const xe = x.endIndex;
                    // i. Let z be the State (xe, cap).
                    const z = new State(xe, cap);
                    // j. Call c(z) and return its result.
                    return c(z);
                };
            }
            case '?!': {
                // 1. Evaluate Disjunction with +1 as its direction argument to obtain a Matcher m.
                const m = Evaluate(Disjunction, +1);
                // 2. Return a new Matcher with parameters (x, c) that captures m and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let d be a new Continuation with parameters (y) that captures nothing and performs the following steps when called:
                    const d = (y) => {
                        // i. Assert: y is a State.
                        Assert(y instanceof State);
                        // ii. Return y.
                        return y;
                    };
                    // d. Call m(x, d) and let r be its result.
                    const r = m(x, d);
                    // e. If r is not failure, return failure.
                    if (r !== 'failure') {
                        return 'failure';
                    }
                    // f. Call c(x) and return its result.
                    return c(x);
                };
            }
            case '?<=': {
                // 1. Evaluate Disjunction with -1 as its direction argument to obtain a Matcher m.
                const m = Evaluate(Disjunction, -1);
                // 2. Return a new Matcher with parameters (x, c) that captures m and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let d be a new Continuation with parameters (y) that captures nothing and performs the following steps when called:
                    const d = (y) => {
                        // i. Assert: y is a State.
                        Assert(y instanceof State);
                        // ii. Return y.
                        return y;
                    };
                    // d. Call m(x, d) and let r be its result.
                    const r = m(x, d);
                    // e. If r is failure, return failure.
                    if (r === 'failure') {
                        return 'failure';
                    }
                    // f. Let y be r's State.
                    const y = r;
                    // g. Let cap be y's captures List.
                    const cap = y.captures;
                    // h. Let xe be x's endIndex.
                    const xe = x.endIndex;
                    // i. Let z be the State (xe, cap).
                    const z = new State(xe, cap);
                    // j. Call c(z) and return its result.
                    return c(z);
                };
            }
            case '?<!': {
                // 1. Evaluate Disjunction with -1 as its direction argument to obtain a Matcher m.
                const m = Evaluate(Disjunction, -1);
                // 2. Return a new Matcher with parameters (x, c) that captures m and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let d be a new Continuation with parameters (y) that captures nothing and performs the following steps when called:
                    const d = (y) => {
                        // i. Assert: y is a State.
                        Assert(y instanceof State);
                        // ii. Return y.
                        return y;
                    };
                    // d. Call m(x, d) and let r be its result.
                    const r = m(x, d);
                    // e. If r is not failure, return failure.
                    if (r !== 'failure') {
                        return 'failure';
                    }
                    // f. Call c(x) and return its result.
                    return c(x);
                };
            }
            default:
                throw new OutOfRange('Evaluate_Assertion', subtype);
        }
    }

    // #sec-runtime-semantics-wordcharacters-abstract-operation
    function WordCharacters() {
        // 1. Let A be a set of characters containing the sixty-three characters:
        //   a b c d e f g h i j k l m n o p q r s t u v w x y z
        //   A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
        //   0 1 2 3 4 5 6 7 8 9 _
        // 2. Let U be an empty set.
        // 3. For each character c not in set A where Canonicalize(c) is in A, add c to U.
        // 4. Assert: Unless Unicode and IgnoreCase are both true, U is empty.
        // 5. Add the characters in set U to set A.
        // Return A.
        const A = new ConcreteCharSet([
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_',
        ].map((c) => c.codePointAt(0)));
        if (Unicode && IgnoreCase) {
            return new VirtualCharSet((c) => {
                if (A.has(c)) {
                    return true;
                }
                if (A.has(Canonicalize(c))) {
                    return true;
                }
                return false;
            });
        }
        return A;
    }

    // #sec-runtime-semantics-iswordchar-abstract-operation
    function IsWordChar(e) {
        // 1. If e is -1 or e is InputLength, return false.
        if (e === -1 || e === InputLength) {
            return false;
        }
        // 2. Let c be the character Input[e].
        const c = Input[e];
        // 3. Let wordChars be the result of ! WordCharacters().
        const wordChars = X(WordCharacters());
        // 4. If c is in wordChars, return true.
        if (wordChars.has(c)) {
            return true;
        }
        // 5. Return false.
        return false;
    }

    // #sec-quantifier
    //   Quantifier ::
    //     QuantifierPrefix
    //     QuantifierPrefix `?`
    function Evaluate_Quantifier({ QuantifierPrefix, greedy }) {
        switch (QuantifierPrefix) {
            case '*':
                return [0, Infinity, greedy];
            case '+':
                return [1, Infinity, greedy];
            case '?':
                return [0, 1, greedy];
            default:
                break;
        }
        const { DecimalDigits_a, DecimalDigits_b } = QuantifierPrefix;
        return [DecimalDigits_a, DecimalDigits_b || DecimalDigits_a, greedy];
    }

    // #sec-atom
    //   Atom ::
    //     PatternCharacter
    //     `.`
    //     `\` AtomEscape
    //     CharacterClass
    //     `(` GroupSpecifier Disjunction `)`
    //     `(` `?` `:` Disjunction `)`
    function Evaluate_Atom(Atom, direction) {
        switch (true) {
            case !!Atom.PatternCharacter: {
                // 1. Let ch be the character matched by PatternCharacter.
                const ch = Atom.PatternCharacter.codePointAt(0);
                // 2. Let A be a one-element CharSet containing the character ch.
                const A = new ConcreteCharSet([Canonicalize(ch)]);
                // 3. Call CharacterSetMatcher(A, false, direction) and return its Matcher result.
                return CharacterSetMatcher(A, false, direction);
            }
            case Atom.subtype === '.': {
                let A;
                // 1. If DotAll is true, then
                if (DotAll) {
                    // a. Let A be the set of all characters.
                    A = new VirtualCharSet((_c) => true);
                } else {
                    // 2. Otherwise, let A be the set of all characters except LineTerminator.
                    A = new VirtualCharSet((c) => !isLineTerminator(String.fromCodePoint(c)));
                }
                // 3. Call CharacterSetMatcher(A, false, direction) and return its Matcher result.
                return CharacterSetMatcher(A, false, direction);
            }
            case !!Atom.CharacterClass: {
                // 1. Evaluate CharacterClass to obtain a CharSet A and a Boolean invert.
                const { A, invert } = Evaluate(Atom.CharacterClass);
                // 2. Call CharacterSetMatcher(A, invert, direction) and return its Matcher result.
                return CharacterSetMatcher(A, invert, direction);
            }
            case Atom.capturing: {
                // 1. Evaluate Disjunction with argument direction to obtain a Matcher m.
                const m = Evaluate(Atom.Disjunction, direction);
                // 2. Let parenIndex be the number of left-capturing parentheses in the entire regular expression
                //    that occur to the left of this Atom. This is the total number of Atom :: `(` GroupSpecifier Disjunction `)`
                //    Parse Nodes prior to or enclosing this Atom.
                const parenIndex = Atom.capturingParenthesesBefore;
                // 3. Return a new Matcher with parameters (x, c) that captures direction, m, and parenIndex and performs the following steps when called:
                return (x, c) => {
                    // a. Assert: x is a State.
                    Assert(x instanceof State);
                    // b. Assert: c is a Continuation.
                    Assert(isContinuation(c));
                    // c. Let d be a new Continuation with parameters (y) that captures x, c, direction, and parenIndex and performs the following steps when called:
                    const d = (y) => {
                        // i. Assert: y is a State.
                        Assert(y instanceof State);
                        // ii. Let cap be a copy of y's captures List.
                        const cap = [...y.captures];
                        // iii. Let xe be x's endIndex.
                        const xe = x.endIndex;
                        // iv. Let ye be y's endIndex.
                        const ye = y.endIndex;
                        let s;
                        // v. If direction is equal to +1, then
                        if (direction === +1) {
                            // 1. Assert: xe ≤ ye.
                            Assert(xe <= ye);
                            if (surroundingAgent.feature('regexp-match-indices')) {
                                // 2. Let r be the Range (xe, ye).
                                s = new Range(xe, ye);
                            } else {
                                // 2. Let s be a new List whose elements are the characters of Input at indices xe (inclusive) through ye (exclusive).
                                s = Input.slice(xe, ye);
                            }
                        } else { // vi. Else,
                            // 1. Assert: direction is equal to -1.
                            Assert(direction === -1);
                            // 2. Assert: ye ≤ xe.
                            Assert(ye <= xe);
                            if (surroundingAgent.feature('regexp-match-indices')) {
                                // 3. Let r be the Range (ye, xe).
                                s = new Range(ye, xe);
                            } else {
                                // 3. Let s be a new List whose elements are the characters of Input at indices ye (inclusive) through xe (exclusive).
                                s = Input.slice(ye, xe);
                            }
                        }
                        // vii. Set cap[parenIndex + 1] to s.
                        cap[parenIndex + 1] = s;
                        // viii. Let z be the State (ye, cap).
                        const z = new State(ye, cap);
                        // ix. Call c(z) and return its result.
                        return c(z);
                    };
                    // d. Call m(x, d) and return its result.
                    return m(x, d);
                };
            }
            case !!Atom.Disjunction:
                return Evaluate(Atom.Disjunction, direction);
            default:
                throw new OutOfRange('Evaluate_Atom', Atom);
        }
    }

    // #sec-runtime-semantics-charactersetmatcher-abstract-operation
    function CharacterSetMatcher(A, invert, direction) {
        // 1. Return a new Matcher with parameters (x, c) that captures A, invert, and direction and performs the following steps when called:
        return (x, c) => {
            // a. Assert: x is a State.
            Assert(x instanceof State);
            // b. Assert: c is a Continuation.
            Assert(isContinuation(c));
            // c. Let e be x's endIndex.
            const e = x.endIndex;
            // d. Let f be e + direction.
            const f = e + direction;
            // e. If f < 0 or f > InputLength, return failure.
            if (f < 0 || f > InputLength) {
                return 'failure';
            }
            // f. Let index be min(e, f).
            const index = Math.min(e, f);
            // g. Let ch be the character Input[index].
            const ch = Input[index];
            // h. Let cc be Canonicalize(ch).
            const cc = Canonicalize(ch);
            // i. If invert is false, then
            if (invert === false) {
                // i. If there does not exist a member a of set A such that Canonicalize(a) is cc, return failure.
                if (!A.has(cc)) {
                    return 'failure';
                }
            } else { // j. Else
                // i. Assert: invert is true.
                Assert(invert === true);
                // ii. If there exists a member a of set A such that Canonicalize(a) is cc, return failure.
                if (A.has(cc)) {
                    return 'failure';
                }
            }
            // k. Let cap be x's captures List.
            const cap = x.captures;
            // Let y be the State (f, cap).
            const y = new State(f, cap);
            // Call c(y) and return its result.
            return c(y);
        };
    }

    // #sec-runtime-semantics-canonicalize-ch
    function Canonicalize(ch) {
        // 1. If IgnoreCase is false, return ch.
        if (IgnoreCase === false) {
            return ch;
        }
        // 2. If Unicode is true, then
        if (Unicode === true) {
            const s = String.fromCodePoint(ch);
            // a. If the file CaseFolding.txt of the Unicode Character Database provides a simple or common case folding mapping for ch, return the result of applying that mapping to ch.
            if (unicodeCaseFoldingSimple.has(s)) {
                return unicodeCaseFoldingSimple.get(s).codePointAt(0);
            }
            if (unicodeCaseFoldingCommon.has(s)) {
                return unicodeCaseFoldingCommon.get(s).codePointAt(0);
            }
            // b. Return ch.
            return ch;
        } else { // 3. Else
            // a. Assert: ch is a UTF-16 code unit.
            // b. Let s be the String value consisting of the single code unit ch.
            const s = String.fromCodePoint(ch);
            // c. Let u be the same result produced as if by performing the algorithm for String.prototype.toUpperCase using s as the this value.
            const u = s.toUpperCase();
            // d. Assert: Type(u) is String.
            Assert(typeof u === 'string');
            // e. If u does not consist of a single code unit, return ch.
            if (u.length !== 1) {
                return ch;
            }
            // f. Let cu be u's single code unit element.
            const cu = u.codePointAt(0);
            // g. If the numeric value of ch ≥ 128 and the numeric value of cu < 128, return ch.
            if (ch >= 128 && cu < 128) {
                return ch;
            }
            // h. Return cu.
            return cu;
        }
    }

    // #sec-atomescape
    // AtomEscape ::
    //   DecimalEscape
    //   CharacterEscape
    //   CharacterClassEscape
    //   `k` GroupName
    function Evaluate_AtomEscape(AtomEscape, direction) {
        switch (true) {
            case !!AtomEscape.DecimalEscape: {
                // 1. Evaluate DecimalEscape to obtain an integer n.
                const n = Evaluate(AtomEscape.DecimalEscape);
                // 2. Assert: n ≤ NcapturingParens.
                Assert(n <= NcapturingParens);
                // 3. Call BackreferenceMatcher(n, direction) and return its Matcher result.
                return BackreferenceMatcher(n, direction);
            }
            case !!AtomEscape.CharacterEscape: {
                // 1. Evaluate CharacterEscape to obtain a character ch.
                const ch = Evaluate(AtomEscape.CharacterEscape);
                // 2. Let A be a one-element CharSet containing the character ch.
                const A = new ConcreteCharSet([Canonicalize(ch)]);
                // 3. Call CharacterSetMatcher(A, false, direction) and return its Matcher result.
                return CharacterSetMatcher(A, false, direction);
            }
            case !!AtomEscape.CharacterClassEscape: {
                // 1. Evaluate CharacterClassEscape to obtain a CharSet A.
                const A = Evaluate(AtomEscape.CharacterClassEscape);
                // 2. Call CharacterSetMatcher(A, false, direction) and return its Matcher result.
                return CharacterSetMatcher(A, false, direction);
            }
            case !!AtomEscape.GroupName: {
                // 1. Search the enclosing Pattern for an instance of a GroupSpecifier for a RegExpIdentifierName which has a StringValue equal to the StringValue of the RegExpIdentifierName contained in GroupName.
                // 2. Assert: A unique such GroupSpecifier is found.
                // 3. Let parenIndex be the number of left-capturing parentheses in the entire regular expression that occur to the left of the located GroupSpecifier. This is the total number of Atom :: `(` GroupSpecifier Disjunction `)` Parse Nodes prior to or enclosing the located GroupSpecifier.
                const parenIndex = Pattern.groupSpecifiers.get(AtomEscape.GroupName);
                Assert(parenIndex !== undefined);
                // 4. Call BackreferenceMatcher(parenIndex, direction) and return its Matcher result.
                return BackreferenceMatcher(parenIndex + 1, direction);
            }
            default:
                throw new OutOfRange('Evaluate_AtomEscape', AtomEscape);
        }
    }

    // #sec-backreference-matcher
    function BackreferenceMatcher(n, direction) {
        // 1. Return a new Matcher with parameters (x, c) that captures n and direction and performs the following steps when called:
        return (x, c) => {
            // a. Assert: x is a State.
            Assert(x instanceof State);
            // b. Assert: c is a Continuation.
            Assert(isContinuation(c));
            // c. Let cap be x's captures List.
            const cap = x.captures;
            // d. Let s be cap[n].
            const s = cap[n];
            // e. If s is undefined, return c(x).
            if (s === Value.undefined) {
                return c(x);
            }
            // f. Let e be x's endIndex.
            const e = x.endIndex;
            let len;
            if (surroundingAgent.feature('regexp-match-indices')) {
                // g. Let rs be r's startIndex.
                const rs = s.startIndex;
                // h. Let re be r's endIndex.
                const re = s.endIndex;
                // i. Let len be the number of elements in re - rs.
                len = re - rs;
            } else {
                // g. Let len be the number of elements in s.
                len = s.length;
            }
            // h. Let f be e + direction × len.
            const f = e + direction * len;
            // i. If f < 0 or f > InputLength, return failure.
            if (f < 0 || f > InputLength) {
                return 'failure';
            }
            // j. Let g be min(e, f).
            const g = Math.min(e, f);
            // k. If there exists an integer i between 0 (inclusive) and len (exclusive) such that Canonicalize(s[i]) is not the same character value as Canonicalize(Input[g + i]), return failure.
            for (let i = 0; i < len; i += 1) {
                const part = surroundingAgent.feature('regexp-match-indices')
                    ? Input[s.startIndex + i]
                    : s[i];
                if (Canonicalize(part) !== Canonicalize(Input[g + i])) {
                    return 'failure';
                }
            }
            // l. Let y be the State (f, cap).
            const y = new State(f, cap);
            // m. Call c(y) and return its result.
            return c(y);
        };
    }

    // #sec-characterescape
    // CharacterEscape ::
    //   ControlEscape
    //   `c` ControlLetter
    //   `0` [lookahead != DecimalDigit]
    //   HexEscapeSequence
    //   RegExpUnicodeEscapeSequence
    //   IdentityEscape
    function Evaluate_CharacterEscape(CharacterEscape) {
        // 1. Let cv be the CharacterValue of this CharacterEscape.
        const cv = CharacterValue(CharacterEscape);
        // 2. Return the character whose character value is cv.
        return cv;
    }

    // #sec-decimalescape
    // DecimalEscape ::
    //   NonZeroDigit DecimalDigits?
    function Evaluate_DecimalEscape(DecimalEscape) {
        return DecimalEscape.value;
    }

    // #sec-characterclassescape
    // CharacterClassEscape ::
    //   `d`
    //   `D`
    //   `s`
    //   `S`
    //   `w`
    //   `W`
    //   `p{` UnicodePropertyValueExpression `}`
    //   `P{` UnicodePropertyValueExpression `}`
    function Evaluate_CharacterClassEscape(node) {
        switch (node.value) {
            case 'd':
                // 1. Return the ten-element set of characters containing the characters 0 through 9 inclusive.
                return new ConcreteCharSet(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].map((c) => c.codePointAt(0)));
            case 'D':
                // 1. Return the set of all characters not included in the set returned by CharacterClassEscape :: `d`.
                return new VirtualCharSet((c) => !isDecimalDigit(String.fromCodePoint(c)));
            case 's':
                // 1. Return the set of characters containing the characters that are on the right-hand side of the WhiteSpace or LineTerminator productions.
                return new VirtualCharSet((c) => {
                    const s = String.fromCodePoint(c);
                    return isWhitespace(s) || isLineTerminator(s);
                });
            case 'S':
                // 1. Return the set of all characters not included in the set returned by CharacterClassEscape :: `s`.
                return new VirtualCharSet((c) => {
                    const s = String.fromCodePoint(c);
                    return !isWhitespace(s) && !isLineTerminator(s);
                });
            case 'w':
                // 1. Return the set of all characters returned by WordCharacters().
                return WordCharacters();
            case 'W': {
                // 1. Return the set of all characters not included in the set returned by CharacterClassEscape :: `w`.
                const s = WordCharacters();
                return new VirtualCharSet((c) => !s.has(c));
            }
            case 'p':
                // 1. Return the CharSet containing all Unicode code points included in the CharSet returned by UnicodePropertyValueExpression.
                return Evaluate(node.UnicodePropertyValueExpression);
            case 'P': {
                // 1. Return the CharSet containing all Unicode code points not included in the CharSet returned by UnicodePropertyValueExpression.
                const s = Evaluate(node.UnicodePropertyValueExpression);
                return new VirtualCharSet((c) => !s.has(c));
            }
            default:
                throw new OutOfRange('Evaluate_CharacterClassEscape', node);
        }
    }

    // UnicodePropertyValueExpression ::
    //   UnicodePropertyName `=` UnicodePropertyValue
    //   LoneUnicodePropertyNameOrValue
    function Evaluate_UnicodePropertyValueExpression(UnicodePropertyValueExpression) {
        if (UnicodePropertyValueExpression.LoneUnicodePropertyNameOrValue) {
            // 1. Let s be SourceText of LoneUnicodePropertyNameOrValue.
            const s = UnicodePropertyValueExpression.LoneUnicodePropertyNameOrValue;
            // 2. If ! UnicodeMatchPropertyValue(General_Category, s) is identical to a List of Unicode code points that is the name of a Unicode general category or general category alias listed in the “Property value and aliases” column of Table 57, then
            if (X(UnicodeMatchPropertyValue('General_Category', s) in UnicodeGeneralCategoryValues)) {
                // a. Return the CharSet containing all Unicode code points whose character database definition includes the property “General_Category” with value s.
                return new ConcreteCharSet(getUnicodePropertyValueSet('General_Category', s));
            }
            // 3. Let p be ! UnicodeMatchProperty(s).
            const p = X(UnicodeMatchProperty(s));
            // 4. Assert: p is a binary Unicode property or binary property alias listed in the “Property name and aliases” column of Table 56.
            Assert(p in BinaryUnicodeProperties);
            // 5. Return the CharSet containing all Unicode code points whose character database definition includes the property p with value “True”.
            return new ConcreteCharSet(getUnicodePropertyValueSet(p));
        }
        // 1. Let ps be SourceText of UnicodePropertyName.
        const ps = UnicodePropertyValueExpression.UnicodePropertyName;
        // 2. Let p be ! UnicodeMatchProperty(ps).
        const p = X(UnicodeMatchProperty(ps));
        // 3. Assert: p is a Unicode property name or property alias listed in the “Property name and aliases” column of Table 55.
        Assert(p in NonbinaryUnicodeProperties);
        // 4. Let vs be SourceText of UnicodePropertyValue.
        const vs = UnicodePropertyValueExpression.UnicodePropertyValue;
        // 5. Let v be ! UnicodeMatchPropertyValue(p, vs).
        const v = X(UnicodeMatchPropertyValue(p, vs));
        // 6. Return the CharSet containing all Unicode code points whose character database definition includes the property p with value v.
        return new ConcreteCharSet(getUnicodePropertyValueSet(p, v));
    }

    // #sec-characterclass
    //  CharacterClass ::
    //    `[` ClassRanges `]`
    //    `[` `^` ClassRanges `]`
    function Evaluate_CharacterClass({ invert, ClassRanges }) {
        let A = new ConcreteCharSet([]);
        for (const range of ClassRanges) {
            if (Array.isArray(range)) {
                const B = Evaluate(range[0]);
                const C = Evaluate(range[1]);
                const D = CharacterRange(B, C);
                A = A.union(D);
            } else {
                A = A.union(Evaluate(range));
            }
        }
        return { A, invert };
    }

    // #sec-runtime-semantics-characterrange-abstract-operation
    function CharacterRange(A, B) {
        // 1. Assert: A and B each contain exactly one character.
        Assert(A.size === 1 && B.size === 1);
        // 2. Let a be the one character in CharSet A.
        const a = A.first();
        // 3. Let b be the one character in CharSet B.
        const b = B.first();
        // 4. Let i be the character value of character a.
        const i = a;
        // 5. Let j be the character value of character b.
        const j = b;
        // 6. Assert: i ≤ j.
        Assert(i <= j);
        // 7. Return the set containing all characters numbered i through j, inclusive.
        const set = new Set();
        for (let k = i; k <= j; k += 1) {
            set.add(Canonicalize(k));
        }
        return new ConcreteCharSet(set);
    }

    // #sec-classatom
    // ClassAtom ::
    //   `-`
    //   ClassAtomNoDash
    // ClassAtomNoDash ::
    //   SourceCharacter
    //   `\` ClassEscape
    function Evaluate_ClassAtom(ClassAtom) {
        switch (true) {
            case !!ClassAtom.SourceCharacter:
                // 1. Return the CharSet containing the character matched by SourceCharacter.
                return new ConcreteCharSet([Canonicalize(ClassAtom.SourceCharacter.codePointAt(0))]);
            case ClassAtom.value === '-':
                // 1. Return the CharSet containing the single character - U+002D (HYPHEN-MINUS).
                return new ConcreteCharSet([0x002D]);
            default:
                throw new OutOfRange('Evaluate_ClassAtom', ClassAtom);
        }
    }

    // #sec-classescape
    // ClassEscape ::
    //   `b`
    //   `-`
    //   CharacterEscape
    //   CharacterClassEscape
    function Evaluate_ClassEscape(ClassEscape) {
        switch (true) {
            case ClassEscape.value === 'b':
            case ClassEscape.value === '-':
            case !!ClassEscape.CharacterEscape: {
                // 1. Let cv be the CharacterValue of this ClassEscape.
                const cv = CharacterValue(ClassEscape);
                // 2. Let c be the character whose character value is cv.
                const c = cv;
                // 3. Return the CharSet containing the single character c.
                return new ConcreteCharSet([Canonicalize(c)]);
            }
            default:
                throw new OutOfRange('Evaluate_ClassEscape', ClassEscape);
        }
    }
}
