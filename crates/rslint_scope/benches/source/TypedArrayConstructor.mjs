import { surroundingAgent } from '../engine.mjs';
import { Type, Value, wellKnownSymbols } from '../value.mjs';
import {
    AllocateArrayBuffer,
    AllocateTypedArray,
    AllocateTypedArrayBuffer,
    Assert,
    CloneArrayBuffer,
    Get,
    GetMethod,
    GetValueFromBuffer,
    IsDetachedBuffer,
    IsSharedArrayBuffer,
    IterableToList,
    SameValue,
    Set,
    SetValueInBuffer,
    SpeciesConstructor,
    LengthOfArrayLike,
    ToIndex,
    ToString,
    typedArrayInfoByName,
} from '../abstract-ops/all.mjs';
import { Q, X } from '../completion.mjs';
import { bootstrapConstructor } from './bootstrap.mjs';

export function BootstrapTypedArrayConstructors(realmRec) {
    Object.entries(typedArrayInfoByName).forEach(([TypedArray, info]) => {
        // #sec-typedarray-constructors
        function TypedArrayConstructor(args, { NewTarget }) {
            if (args.length === 0) {
                // #sec-typedarray
                // 1. If NewTarget is undefined, throw a TypeError exception.
                if (NewTarget === Value.undefined) {
                    return surroundingAgent.Throw('TypeError', 'ConstructorNonCallable', this);
                }
                // 2. Let constructorName be the String value of the Constructor Name value specified in Table 61 for this TypedArray constructor.
                const constructorName = new Value(TypedArray);
                // 3. Return ? AllocateTypedArray(constructorName, NewTarget, "%TypedArray.prototype%", 0).
                return Q(AllocateTypedArray(constructorName, NewTarget, `%${TypedArray}.prototype%`, new Value(0)));
            } else if (Type(args[0]) !== 'Object') {
                // #sec-typedarray-length
                const [length] = args;
                // 1. Assert: Type(length) is not Object.
                Assert(Type(length) !== 'Object');
                // 2. If NewTarget is undefined, throw a TypeError exception.
                if (NewTarget === Value.undefined) {
                    return surroundingAgent.Throw('TypeError', 'ConstructorNonCallable', this);
                }
                // 3. Let elementLength be ? ToIndex(length).
                const elementLength = Q(ToIndex(length));
                // 4. Let constructorName be the String value of the Constructor Name value specified in Table 61 for this TypedArray constructor.
                const constructorName = new Value(TypedArray);
                // 5. Return ? AllocateTypedArray(constructorName, NewTarget, "%TypedArray.prototype%", elementLength).
                return Q(AllocateTypedArray(constructorName, NewTarget, `%${TypedArray}.prototype%`, elementLength));
            } else if ('TypedArrayName' in args[0]) {
                // #sec-typedarray-typedarray
                const [typedArray] = args;
                // 1. Assert: Type(typedArray) is Object and typedArray has a [[TypedArrayName]] internal slot.
                Assert(Type(typedArray) === 'Object' && 'TypedArrayName' in typedArray);
                // 2. If NewTarget is undefined, throw a TypeError exception.
                if (NewTarget === Value.undefined) {
                    return surroundingAgent.Throw('TypeError', 'ConstructorNonCallable', this);
                }
                // 3. Let constructorName be the String value of the Constructor Name value specified in Table 61 for this TypedArray constructor.
                const constructorName = new Value(TypedArray);
                // 4. Let O be ? AllocateTypedArray(constructorName, NewTarget, "%TypedArray.prototype%").
                const O = Q(AllocateTypedArray(constructorName, NewTarget, `%${TypedArray}.prototype%`));
                // 5. Let srcArray be typedArray.
                const srcArray = typedArray;
                // 6. Let srcData be srcArray.[[ViewedArrayBuffer]].
                const srcData = srcArray.ViewedArrayBuffer;
                // 7. If IsDetachedBuffer(srcData) is true, throw a TypeError exception.
                if (IsDetachedBuffer(srcData) === Value.true) {
                    return surroundingAgent.Throw('TypeError', 'ArrayBufferDetached');
                }
                // 8. Let elementType be the Element Type value in Table 61 for constructorName.
                const elementType = new Value(info.ElementType);
                // 9. Let elementLength be srcArray.[[ArrayLength]].
                const elementLength = srcArray.ArrayLength;
                // 10. Let srcName be the String value of srcArray.[[TypedArrayName]].
                const srcName = srcArray.TypedArrayName.stringValue();
                // 11. Let srcType be the Element Type value in Table 61 for srcName.
                const srcType = new Value(typedArrayInfoByName[srcName].ElementType);
                // 12. Let srcElementSize be the Element Size value specified in Table 61 for srcName.
                const srcElementSize = typedArrayInfoByName[srcName].ElementSize;
                // 13. Let srcByteOffset be srcArray.[[ByteOffset]].
                const srcByteOffset = srcArray.ByteOffset;
                // 14. Let elementSize be the Element Size value specified in Table 61 for constructorName.
                const elementSize = info.ElementSize;
                // 15. Let byteLength be elementSize × elementLength.
                const byteLength = new Value(elementSize * elementLength.numberValue());
                // 16. If IsSharedArrayBuffer(srcData) is false, then
                let bufferConstructor;
                if (IsSharedArrayBuffer(srcData) === Value.false) {
                    bufferConstructor = Q(SpeciesConstructor(srcData, surroundingAgent.intrinsic('%ArrayBuffer%')));
                } else {
                    // 17. Else, Let bufferConstructor be %ArrayBuffer%.
                    bufferConstructor = surroundingAgent.intrinsic('%ArrayBuffer%');
                }
                // 18. If elementType is the same as srcType, then
                let data;
                if (SameValue(elementType, srcType) === Value.true) {
                    // a. Let data be ? CloneArrayBuffer(srcData, srcByteOffset, byteLength, bufferConstructor).
                    data = Q(CloneArrayBuffer(srcData, srcByteOffset, byteLength, bufferConstructor));
                } else {
                    // a. Let data be ? AllocateArrayBuffer(bufferConstructor, byteLength).
                    data = Q(AllocateArrayBuffer(bufferConstructor, byteLength));
                    // b. If IsDetachedBuffer(srcData) is true, throw a TypeError exception.
                    if (IsDetachedBuffer(srcData) === Value.true) {
                        return surroundingAgent.Throw('TypeError', 'ArrayBufferDetached');
                    }
                    // c. If srcArray.[[ContentType]] is not equal to O.[[ContentType]], throw a TypeError exception.
                    if (srcArray.ContentType !== O.ContentType) {
                        return surroundingAgent.Throw('TypeError', 'BufferContentTypeMismatch');
                    }
                    // d. Let srcByteIndex be srcByteOffset.
                    let srcByteIndex = srcByteOffset.numberValue();
                    // e. Let targetByteIndex be 0.
                    let targetByteIndex = 0;
                    // f. Let count be elementLength.
                    let count = elementLength.numberValue();
                    // g. Repeat, while count > 0
                    while (count > 0) {
                        // i. Let value be GetValueFromBuffer(srcData, srcByteIndex, srcType, true, Unordered).
                        const value = GetValueFromBuffer(srcData, new Value(srcByteIndex), srcType.stringValue(), true, 'Unordered');
                        // ii. Perform SetValueInBuffer(data, targetByteIndex, elementType, value, true, Unordered).
                        SetValueInBuffer(data, new Value(targetByteIndex), elementType.stringValue(), value, true, 'Unordered');
                        // iii. Set srcByteIndex to srcByteIndex + srcElementSize.
                        srcByteIndex += srcElementSize;
                        // iv. Set targetByteIndex to targetByteIndex + elementSize.
                        targetByteIndex += elementSize;
                        // v. Set count to count - 1.
                        count -= 1;
                    }
                }
                // 20. Set O.[[ViewedArrayBuffer]] to data.
                O.ViewedArrayBuffer = data;
                // 21. Set O.[[ByteLength]] to byteLength.
                O.ByteLength = byteLength;
                // 22. Set O.[[ByteOffset]] to 0.
                O.ByteOffset = new Value(0);
                // 23. Set O.[[ArrayLength]] to elementLength.
                O.ArrayLength = elementLength;
                // 24. Return O.
                return O;
            } else if (!('TypedArrayName' in args[0]) && !('ArrayBufferData' in args[0])) {
                // 22.2.4.4 #sec-typedarray-object
                const [object] = args;
                // 1. Assert: Type(object) is Object and object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]] internal slot.
                Assert(Type(object) === 'Object' && !('TypedArrayName' in object) && !('ArrayBufferData' in object));
                // 2. If NewTarget is undefined, throw a TypeError exception.
                if (NewTarget === Value.undefined) {
                    return surroundingAgent.Throw('TypeError', 'ConstructorNonCallable', this);
                }
                // 3. Let constructorName be the String value of the Constructor Name value specified in Table 61 for this TypedArray constructor.
                const constructorName = new Value(TypedArray);
                // 4. Let O be ? AllocateTypedArray(constructorName, NewTarget, "%TypedArray.prototype%").
                const O = Q(AllocateTypedArray(constructorName, NewTarget, `%${TypedArray}.prototype%`));
                // 5. Let usingIterator be ? GetMethod(object, @@iterator).
                const usingIterator = Q(GetMethod(object, wellKnownSymbols.iterator));
                // 6. If usingIterator is not undefined, then
                if (usingIterator !== Value.undefined) {
                    // a. Let values be ? IterableToList(object, usingIterator).
                    const values = Q(IterableToList(object, usingIterator));
                    // b. Let len be the number of elements in values.
                    const len = values.length;
                    // c. Perform ? AllocateTypedArrayBuffer(O, len).
                    Q(AllocateTypedArrayBuffer(O, new Value(len)));
                    // d. Let k be 0.
                    let k = 0;
                    // e. Repeat, while k < len
                    while (k < len) {
                        // i. Let Pk be ! ToString(k).
                        const Pk = X(ToString(new Value(k)));
                        // ii. Let kValue be the first element of values and remove that element from values.
                        const kValue = values.shift();
                        // iii. Perform ? Set(O, Pk, kValue, true).
                        Q(Set(O, Pk, kValue, Value.true));
                        // iv. Set k to k + 1.
                        k += 1;
                    }
                    // f. Assert: values is now an empty List.
                    Assert(values.length === 0);
                    // g. Return O.
                    return O;
                }
                // 7. NOTE: object is not an Iterable so assume it is already an array-like object.
                // 8. Let arrayLike be object.
                const arrayLike = object;
                // 9. Let len be ? LengthOfArrayLike(arrayLike).
                const len = Q(LengthOfArrayLike(arrayLike)).numberValue();
                // 10. Perform ? AllocateTypedArrayBuffer(O, len).
                Q(AllocateTypedArrayBuffer(O, new Value(len)));
                // 11. Let k be 0.
                let k = 0;
                // 12. Repeat, while k < len.
                while (k < len) {
                    // a. Let Pk be ! ToString(k).
                    const Pk = X(ToString(new Value(k)));
                    // b. Let kValue be ? Get(arrayLike, Pk).
                    const kValue = Q(Get(arrayLike, Pk));
                    // c. Perform ? Set(O, Pk, kValue, true).
                    Q(Set(O, Pk, kValue, Value.true));
                    // d. Set k to k + 1.
                    k += 1;
                }
                // 13. Return O.
                return O;
            } else {
                // #sec-typedarray-buffer-byteoffset-length
                const [buffer = Value.undefined, byteOffset = Value.undefined, length = Value.undefined] = args;
                // 1. Assert: Type(buffer) is Object and buffer has an [[ArrayBufferData]] internal slot.
                Assert(Type(buffer) === 'Object' && 'ArrayBufferData' in buffer);
                // 2. If NewTarget is undefined, throw a TypeError exception.
                if (NewTarget === Value.undefined) {
                    return surroundingAgent.Throw('TypeError', 'ConstructorNonCallable', this);
                }
                // 3. Let constructorName be the String value of the Constructor Name value specified in Table 61 for this TypedArray constructor.
                const constructorName = new Value(TypedArray);
                // 4. Let O be ? AllocateTypedArray(constructorName, NewTarget, "%TypedArray.prototype%").
                const O = Q(AllocateTypedArray(constructorName, NewTarget, `%${TypedArray}.prototype%`));
                // 5. Let elementSize be the Element Size value specified in Table 61 for constructorName.
                const elementSize = info.ElementSize;
                // 6. Let offset be ? ToIndex(byteOffset).
                const offset = Q(ToIndex(byteOffset));
                // 7. If offset modulo elementSize ≠ 0, throw a RangeError exception.
                if (offset.numberValue() % elementSize !== 0) {
                    return surroundingAgent.Throw('RangeError', 'TypedArrayOffsetAlignment', TypedArray, elementSize);
                }
                // 8. If length is not undefined, then
                let newLength;
                if (length !== Value.undefined) {
                    // Let newLength be ? ToIndex(length).
                    newLength = Q(ToIndex(length)).numberValue();
                }
                // 9. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
                if (IsDetachedBuffer(buffer) === Value.true) {
                    return surroundingAgent.Throw('TypeError', 'ArrayBufferDetached');
                }
                // 10. Let bufferByteLength be buffer.[[ArrayBufferByteLength]].
                const bufferByteLength = buffer.ArrayBufferByteLength.numberValue();
                // 11. If length is undefined, then
                let newByteLength;
                if (length === Value.undefined) {
                    // a. If bufferByteLength modulo elementSize ≠ 0, throw a RangeError exception.
                    if (bufferByteLength % elementSize !== 0) {
                        return surroundingAgent.Throw('RangeError', 'TypedArrayLengthAlignment', TypedArray, elementSize);
                    }
                    // b. Let newByteLength be bufferByteLength - offset.
                    newByteLength = bufferByteLength - offset.numberValue();
                    // c. If newByteLength < 0, throw a RangeError exception.
                    if (newByteLength < 0) {
                        return surroundingAgent.Throw('RangeError', 'TypedArrayCreationOOB');
                    }
                } else {
                    // a. Let newByteLength be newLength × elementSize.
                    newByteLength = newLength * elementSize;
                    // b. If offset + newByteLength > bufferByteLength, throw a RangeError exception.
                    if (offset.numberValue() + newByteLength > bufferByteLength) {
                        return surroundingAgent.Throw('RangeError', 'TypedArrayCreationOOB');
                    }
                }
                // 13. Set O.[[ViewedArrayBuffer]] to buffer.
                O.ViewedArrayBuffer = buffer;
                // 14. Set O.[[ByteLength]] to newByteLength.
                O.ByteLength = new Value(newByteLength);
                // 15. Set O.[[ByteOffset]] to offset.
                O.ByteOffset = offset;
                // 16. Set O.[[ArrayLength]] to newByteLength / elementSize.
                O.ArrayLength = new Value(newByteLength / elementSize);
                // 17. Return O.
                return O;
            }
        }

        const taConstructor = bootstrapConstructor(realmRec, TypedArrayConstructor, TypedArray, 3, realmRec.Intrinsics[`%${TypedArray}.prototype%`], [
            ['BYTES_PER_ELEMENT', new Value(info.ElementSize), undefined, {
                Writable: Value.false,
                Configurable: Value.false,
            }],
        ]);
        X(taConstructor.SetPrototypeOf(realmRec.Intrinsics['%TypedArray%']));
        realmRec.Intrinsics[`%${TypedArray}%`] = taConstructor;
    });
}
