import {
    Assert,
    Call,
    DeletePropertyOrThrow,
    Get,
    HasOwnProperty,
    HasProperty,
    Invoke,
    IsCallable,
    SameValueZero,
    Set,
    StrictEqualityComparison,
    ToBoolean,
    ToInteger,
    ToLength,
    ToObject,
    ToString,
} from '../abstract-ops/all.mjs';
import { Q, X } from '../completion.mjs';
import { surroundingAgent } from '../engine.mjs';
import { Type, Value } from '../value.mjs';
import { assignProps } from './bootstrap.mjs';

// Algorithms and methods shared between %Array.prototype% and
// %TypedArray.prototype%.

// 22.1.3.27 #sec-array.prototype.sort
// 22.2.3.26 #sec-%typedarray%.prototype.sort
//
// If internalMethodsRestricted is true, then Asserts are used to ensure that
// "The only internal methods of the this object that the algorithm may call
// are [[Get]] and [[Set]]," a requirement of %TypedArray%.prototype.sort.
export function ArrayProto_sortBody(obj, len, SortCompare, internalMethodsRestricted = false) {
    len = len.numberValue();

    // Collect all elements. Count how many holes we have for error checking.
    const collected = [];
    let holes = 0;
    for (let k = 0; k < len; k += 1) {
        const curProp = X(ToString(new Value(k)));
        const prop = Q(Get(obj, curProp));
        if (prop === Value.undefined) {
            Assert(!internalMethodsRestricted);
            const hasOwn = Q(HasOwnProperty(obj, curProp));
            if (hasOwn === Value.false) {
                holes += 1;
            } else {
                collected.push(prop);
            }
        } else {
            collected.push(prop);
        }
    }
    if (internalMethodsRestricted) {
        Assert(holes === 0);
    }
    Assert(collected.length + holes === len);

    // Get rid of holes by deleting properties at the end.
    // See Note 1: Because non-existent property values always compare greater
    // than undefined property values, and undefined always compares greater
    // than any other value, undefined property values always sort to the end
    // of the result, followed by non-existent property values.
    for (let k = collected.length; k < len; k += 1) {
        const curProp = X(ToString(new Value(k)));
        Q(DeletePropertyOrThrow(obj, curProp));
    }

    // Mergesort.
    const lBuffer = [];
    const rBuffer = [];
    for (let step = 1; step < collected.length; step *= 2) {
        for (let start = 0; start < collected.length - 1; start += 2 * step) {
            const sizeLeft = step;
            const mid = start + sizeLeft;
            const sizeRight = Math.min(step, collected.length - mid);
            if (sizeRight < 0) {
                continue;
            }

            // Merge.
            for (let l = 0; l < sizeLeft; l += 1) {
                lBuffer[l] = collected[start + l];
            }
            for (let r = 0; r < sizeRight; r += 1) {
                rBuffer[r] = collected[mid + r];
            }

            {
                let l = 0;
                let r = 0;
                let o = start;
                while (l < sizeLeft && r < sizeRight) {
                    const cmp = Q(SortCompare(lBuffer[l], rBuffer[r])).numberValue();
                    if (cmp <= 0) {
                        collected[o] = lBuffer[l];
                        o += 1;
                        l += 1;
                    } else {
                        collected[o] = rBuffer[r];
                        o += 1;
                        r += 1;
                    }
                }
                while (l < sizeLeft) {
                    collected[o] = lBuffer[l];
                    o += 1;
                    l += 1;
                }
                while (r < sizeRight) {
                    collected[o] = rBuffer[r];
                    o += 1;
                    r += 1;
                }
            }
        }
    }

    // Copy the sorted results back to the array.
    for (let k = 0; k < collected.length; k += 1) {
        const curProp = X(ToString(new Value(k)));
        Q(Set(obj, curProp, collected[k], Value.true));
    }

    return obj;
}

export function BootstrapArrayPrototypeShared(realmRec, proto, priorToEvaluatingAlgorithm, objectToLength) {
    // 22.1.3.5 #sec-array.prototype.every
    // 22.2.3.7 #sec-%typedarray%.prototype.every
    function ArrayProto_every([callbackFn = Value.undefined, thisArg = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp));
        if (IsCallable(callbackFn) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', callbackFn);
        }
        let k = 0;
        while (k < len.numberValue()) {
            const Pk = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, Pk));
            if (kPresent === Value.true) {
                const kValue = Q(Get(O, Pk));
                const testResult = ToBoolean(Q(Call(callbackFn, thisArg, [kValue, new Value(k), O])));
                if (testResult === Value.false) {
                    return Value.false;
                }
            }
            k += 1;
        }
        return Value.true;
    }

    // 22.1.3.8 #sec-array.prototype.find
    // 22.2.3.10 #sec-%typedarray%.prototype.find
    function ArrayProto_find([predicate = Value.undefined, thisArg = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(predicate) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', predicate);
        }
        let k = 0;
        while (k < len) {
            const Pk = X(ToString(new Value(k)));
            const kValue = Q(Get(O, Pk));
            const testResult = ToBoolean(Q(Call(predicate, thisArg, [kValue, new Value(k), O])));
            if (testResult === Value.true) {
                return kValue;
            }
            k += 1;
        }
        return Value.undefined;
    }

    // 22.1.3.9 #sec-array.prototype.findindex
    // 22.2.3.11 #sec-%typedarray%.prototype.findindex
    function ArrayProto_findIndex([predicate = Value.undefined, thisArg = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(predicate) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', predicate);
        }
        let k = 0;
        while (k < len) {
            const Pk = X(ToString(new Value(k)));
            const kValue = Q(Get(O, Pk));
            const testResult = ToBoolean(Q(Call(predicate, thisArg, [kValue, new Value(k), O])));
            if (testResult === Value.true) {
                return new Value(k);
            }
            k += 1;
        }
        return new Value(-1);
    }

    // 22.1.3.12 #sec-array.prototype.foreach
    // 22.2.3.12 #sec-%typedarray%.prototype.foreach
    function ArrayProto_forEach([callbackfn = Value.undefined, thisArg = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(callbackfn) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', callbackfn);
        }
        let k = 0;
        while (k < len) {
            const Pk = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, Pk));
            if (kPresent === Value.true) {
                const kValue = Q(Get(O, Pk));
                Q(Call(callbackfn, thisArg, [kValue, new Value(k), O]));
            }
            k += 1;
        }
        return Value.undefined;
    }

    // 22.1.3.13 #sec-array.prototype.includes
    // 22.2.3.13 #sec-%typedarray%.prototype.includes
    function ArrayProto_includes([searchElement = Value.undefined, fromIndex = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (len === 0) {
            return Value.false;
        }
        const n = Q(ToInteger(fromIndex)).numberValue();
        if (fromIndex === Value.undefined) {
            Assert(n === 0);
        }
        let k;
        if (n >= 0) {
            k = n;
        } else {
            k = len + n;
            if (k < 0) {
                k = 0;
            }
        }
        while (k < len) {
            const kStr = X(ToString(new Value(k)));
            const elementK = Q(Get(O, kStr));
            if (SameValueZero(searchElement, elementK) === Value.true) {
                return Value.true;
            }
            k += 1;
        }
        return Value.false;
    }

    // 22.1.3.14 #sec-array.prototype.indexof
    // 22.2.3.14 #sec-%typedarray%.prototype.indexof
    function ArrayProto_indexOf([searchElement = Value.undefined, fromIndex = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (len === 0) {
            return new Value(-1);
        }
        const n = Q(ToInteger(fromIndex)).numberValue();
        if (fromIndex === Value.undefined) {
            Assert(n === 0);
        }
        if (n >= len) {
            return new Value(-1);
        }
        let k;
        if (n >= 0) {
            k = n;
        } else {
            k = len + n;
            if (k < 0) {
                k = 0;
            }
        }
        while (k < len) {
            const kStr = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, kStr));
            if (kPresent === Value.true) {
                const elementK = Q(Get(O, kStr));
                const same = StrictEqualityComparison(searchElement, elementK);
                if (same === Value.true) {
                    return new Value(k);
                }
            }
            k += 1;
        }
        return new Value(-1);
    }

    // 22.1.3.15 #sec-array.prototype.join
    // 22.2.3.15 #sec-%typedarray%.prototype.join
    function ArrayProto_join([separator = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        let sep;
        if (Type(separator) === 'Undefined') {
            sep = ',';
        } else {
            sep = Q(ToString(separator)).stringValue();
        }
        let R = '';
        let k = 0;
        while (k < len) {
            if (k > 0) {
                R = `${R}${sep}`;
            }
            const kStr = X(ToString(new Value(k)));
            const element = Q(Get(O, kStr));
            let next;
            if (Type(element) === 'Undefined' || Type(element) === 'Null') {
                next = '';
            } else {
                next = Q(ToString(element)).stringValue();
            }
            R = `${R}${next}`;
            k += 1;
        }
        return new Value(R);
    }

    // 22.1.3.17 #sec-array.prototype.lastindexof
    // 22.2.3.17 #sec-%typedarray%.prototype.lastindexof
    function ArrayProto_lastIndexOf([searchElement = Value.undefined, fromIndex], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (len === 0) {
            return new Value(-1);
        }
        let n;
        if (fromIndex !== undefined) {
            n = Q(ToInteger(fromIndex)).numberValue();
        } else {
            n = len - 1;
        }
        let k;
        if (n >= 0) {
            k = Math.min(n, len - 1);
        } else {
            k = len + n;
        }
        while (k >= 0) {
            const kStr = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, kStr));
            if (kPresent === Value.true) {
                const elementK = Q(Get(O, kStr));
                const same = StrictEqualityComparison(searchElement, elementK);
                if (same === Value.true) {
                    return new Value(k);
                }
            }
            k -= 1;
        }
        return new Value(-1);
    }

    // 22.1.3.21 #sec-array.prototype.reduce
    // 22.2.3.20 #sec-%typedarray%.prototype.reduce
    function ArrayProto_reduce([callbackfn = Value.undefined, initialValue], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(callbackfn) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', callbackfn);
        }
        if (len === 0 && initialValue === undefined) {
            return surroundingAgent.Throw('TypeError', 'ArrayEmptyReduce');
        }
        let k = 0;
        let accumulator = Value.undefined;
        if (initialValue !== undefined) {
            accumulator = initialValue;
        } else {
            let kPresent = false;
            while (kPresent === false && k < len) {
                const Pk = X(ToString(new Value(k)));
                kPresent = Q(HasProperty(O, Pk)) === Value.true;
                if (kPresent === true) {
                    accumulator = Q(Get(O, Pk));
                }
                k += 1;
            }
            if (kPresent === false) {
                return surroundingAgent.Throw('TypeError', 'ArrayEmptyReduce');
            }
        }
        while (k < len) {
            const Pk = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, Pk));
            if (kPresent === Value.true) {
                const kValue = Q(Get(O, Pk));
                accumulator = Q(Call(callbackfn, Value.undefined, [accumulator, kValue, new Value(k), O]));
            }
            k += 1;
        }
        return accumulator;
    }

    // 22.1.3.22 #sec-array.prototype.reduceright
    // 22.2.3.21 #sec-%typedarray%.prototype.reduceright
    function ArrayProto_reduceRight([callbackfn = Value.undefined, initialValue], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(callbackfn) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', callbackfn);
        }
        if (len === 0 && initialValue === undefined) {
            return surroundingAgent.Throw('TypeError', 'ArrayEmptyReduce');
        }
        let k = len - 1;
        let accumulator = Value.undefined;
        if (initialValue !== undefined) {
            accumulator = initialValue;
        } else {
            let kPresent = false;
            while (kPresent === false && k >= 0) {
                const Pk = X(ToString(new Value(k)));
                kPresent = Q(HasProperty(O, Pk)) === Value.true;
                if (kPresent === true) {
                    accumulator = Q(Get(O, Pk));
                }
                k -= 1;
            }
            if (kPresent === false) {
                return surroundingAgent.Throw('TypeError', 'ArrayEmptyReduce');
            }
        }
        while (k >= 0) {
            const Pk = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, Pk));
            if (kPresent === Value.true) {
                const kValue = Q(Get(O, Pk));
                accumulator = Q(Call(callbackfn, Value.undefined, [accumulator, kValue, new Value(k), O]));
            }
            k -= 1;
        }
        return accumulator;
    }

    // 22.1.3.23 #sec-array.prototype.reverse
    // 22.2.3.22 #sec-%typedarray%.prototype.reverse
    function ArrayProto_reverse(args, { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        const middle = Math.floor(len / 2);
        let lower = 0;
        while (lower !== middle) {
            const upper = len - lower - 1;
            const upperP = X(ToString(new Value(upper)));
            const lowerP = X(ToString(new Value(lower)));
            const lowerExists = Q(HasProperty(O, lowerP));
            let lowerValue;
            let upperValue;
            if (lowerExists === Value.true) {
                lowerValue = Q(Get(O, lowerP));
            }
            const upperExists = Q(HasProperty(O, upperP));
            if (upperExists === Value.true) {
                upperValue = Q(Get(O, upperP));
            }
            if (lowerExists === Value.true && upperExists === Value.true) {
                Q(Set(O, lowerP, upperValue, Value.true));
                Q(Set(O, upperP, lowerValue, Value.true));
            } else if (lowerExists === Value.false && upperExists === Value.true) {
                Q(Set(O, lowerP, upperValue, Value.true));
                Q(DeletePropertyOrThrow(O, upperP));
            } else if (lowerExists === Value.true && upperExists === Value.false) {
                Q(DeletePropertyOrThrow(O, lowerP));
                Q(Set(O, upperP, lowerValue, Value.true));
            } else {
                // no further action is required
            }
            lower += 1;
        }
        return O;
    }

    // 22.1.3.26 #sec-array.prototype.some
    // 22.2.3.25 #sec-%typedarray%.prototype.some
    function ArrayProto_some([callbackfn = Value.undefined, thisArg = Value.undefined], { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const O = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(O));
        const len = Q(ToLength(lenProp)).numberValue();
        if (IsCallable(callbackfn) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'NotAFunction', callbackfn);
        }
        let k = 0;
        while (k < len) {
            const Pk = X(ToString(new Value(k)));
            const kPresent = Q(HasProperty(O, Pk));
            if (kPresent === Value.true) {
                const kValue = Q(Get(O, Pk));
                const testResult = ToBoolean(Q(Call(callbackfn, thisArg, [kValue, new Value(k), O])));
                if (testResult === Value.true) {
                    return Value.true;
                }
            }
            k += 1;
        }
        return Value.false;
    }

    // 22.1.3.29 #sec-array.prototype.tolocalestring
    // 22.2.3.28 #sec-%typedarray%.prototype.tolocalestring
    function ArrayProto_toLocaleString(args, { thisValue }) {
        Q(priorToEvaluatingAlgorithm(thisValue));
        const array = Q(ToObject(thisValue));
        const lenProp = Q(objectToLength(array));
        const len = Q(ToLength(lenProp)).numberValue();
        const separator = ', ';
        let R = '';
        let k = 0;
        while (k < len) {
            if (k > 0) {
                R = `${R}${separator}`;
            }
            const kStr = X(ToString(new Value(k)));
            const nextElement = Q(Get(array, kStr));
            if (nextElement !== Value.undefined && nextElement !== Value.null) {
                const S = Q(ToString(Q(Invoke(nextElement, new Value('toLocaleString'))))).stringValue();
                R = `${R}${S}`;
            }
            k += 1;
        }
        return new Value(R);
    }

    assignProps(realmRec, proto, [
        ['every', ArrayProto_every, 1],
        ['find', ArrayProto_find, 1],
        ['findIndex', ArrayProto_findIndex, 1],
        ['forEach', ArrayProto_forEach, 1],
        ['includes', ArrayProto_includes, 1],
        ['indexOf', ArrayProto_indexOf, 1],
        ['join', ArrayProto_join, 1],
        ['lastIndexOf', ArrayProto_lastIndexOf, 1],
        ['reduce', ArrayProto_reduce, 1],
        ['reduceRight', ArrayProto_reduceRight, 1],
        ['reverse', ArrayProto_reverse, 0],
        ['some', ArrayProto_some, 1],
        ['toLocaleString', ArrayProto_toLocaleString, 0],
    ]);
}
