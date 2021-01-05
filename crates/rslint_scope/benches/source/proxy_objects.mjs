import { surroundingAgent } from '../engine.mjs';
import { Type, Value } from '../value.mjs';
import { Q, X } from '../completion.mjs';
import { ValueSet } from '../helpers.mjs';
import {
    Assert,
    MakeBasicObject,
    IsConstructor,
    IsCallable,
    Call,
    Construct,
    GetMethod,
    CreateArrayFromList,
    CreateListFromArrayLike,
    IsExtensible,
    IsPropertyKey,
    SameValue,
    ToBoolean,
    ToPropertyDescriptor,
    FromPropertyDescriptor,
    CompletePropertyDescriptor,
    IsCompatiblePropertyDescriptor,
    IsDataDescriptor,
    IsAccessorDescriptor,
} from './all.mjs';

// #sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
function ProxyGetPrototypeOf() {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'getPrototypeOf');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('getPrototypeOf')));
    if (trap === Value.undefined) {
        return Q(target.GetPrototypeOf());
    }
    const handlerProto = Q(Call(trap, handler, [target]));
    if (Type(handlerProto) !== 'Object' && Type(handlerProto) !== 'Null') {
        return surroundingAgent.Throw('TypeError', 'ProxyGetPrototypeOfInvalid');
    }
    const extensibleTarget = Q(IsExtensible(target));
    if (extensibleTarget === Value.true) {
        return handlerProto;
    }
    const targetProto = Q(target.GetPrototypeOf());
    if (SameValue(handlerProto, targetProto) === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxyGetPrototypeOfNonExtensible');
    }
    return handlerProto;
}

// #sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
function ProxySetPrototypeOf(V) {
    const O = this;

    Assert(Type(V) === 'Object' || Type(V) === 'Null');
    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'setPrototypeOf');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('setPrototypeOf')));
    if (trap === Value.undefined) {
        return Q(target.SetPrototypeOf(V));
    }
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target, V])));
    if (booleanTrapResult === Value.false) {
        return Value.false;
    }
    const extensibleTarget = Q(IsExtensible(target));
    if (extensibleTarget === Value.true) {
        return Value.true;
    }
    const targetProto = Q(target.GetPrototypeOf());
    if (SameValue(V, targetProto) === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxySetPrototypeOfNonExtensible');
    }
    return Value.true;
}

// #sec-proxy-object-internal-methods-and-internal-slots-isextensible
function ProxyIsExtensible() {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'isExtensible');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('isExtensible')));
    if (trap === Value.undefined) {
        return Q(IsExtensible(target));
    }
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target])));
    const targetResult = Q(IsExtensible(target));
    if (SameValue(booleanTrapResult, targetResult) === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxyIsExtensibleInconsistent', targetResult);
    }
    return booleanTrapResult;
}

// #sec-proxy-object-internal-methods-and-internal-slots-preventextensions
function ProxyPreventExtensions() {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'preventExtensions');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('preventExtensions')));
    if (trap === Value.undefined) {
        return Q(target.PreventExtensions());
    }
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target])));
    if (booleanTrapResult === Value.true) {
        const extensibleTarget = Q(IsExtensible(target));
        if (extensibleTarget === Value.true) {
            return surroundingAgent.Throw('TypeError', 'ProxyPreventExtensionsExtensible');
        }
    }
    return booleanTrapResult;
}

// #sec-proxy-object-internal-methods-and-internal-slots-getownproperty-p
function ProxyGetOwnProperty(P) {
    const O = this;

    // 1. Assert: IsPropertyKey(P) is true.
    Assert(IsPropertyKey(P));
    // 2. Let handler be O.[[ProxyHandler]].
    const handler = O.ProxyHandler;
    // 3. If handler is null, throw a TypeError exception.
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'getOwnPropertyDescriptor');
    }
    // 4. Assert: Type(Handler) is Object.
    Assert(Type(handler) === 'Object');
    // 5. Let target be O.[[ProxyTarget]].
    const target = O.ProxyTarget;
    // 6. Let trap be ? Getmethod(handler, "getOwnPropertyDescriptor").
    const trap = Q(GetMethod(handler, new Value('getOwnPropertyDescriptor')));
    // 7. If trap is undefined, then
    if (trap === Value.undefined) {
        // a. Return ? target.[[GetOwnProperty]](P).
        return Q(target.GetOwnProperty(P));
    }
    // 8. Let trapResultObj be ? Call(trap, handler, « target, P »).
    const trapResultObj = Q(Call(trap, handler, [target, P]));
    // 9. If Type(trapResultObj) is neither Object nor Undefined, throw a TypeError exception.
    if (Type(trapResultObj) !== 'Object' && Type(trapResultObj) !== 'Undefined') {
        return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorInvalid', P);
    }
    // 10. Let targetDesc be ? target.[[GetOwnProperty]](P).
    const targetDesc = Q(target.GetOwnProperty(P));
    // 11. If trapResultObj is undefined, then
    if (trapResultObj === Value.undefined) {
        // a. If targetDesc is undefined, return undefined.
        if (targetDesc === Value.undefined) {
            return Value.undefined;
        }
        // b. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
        if (targetDesc.Configurable === Value.false) {
            return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorUndefined', P);
        }
        // c. Let extensibleTarget be ? IsExtensible(target).
        const extensibleTarget = Q(IsExtensible(target));
        // d. If extensibleTarget is false, throw a TypeError exception.
        if (extensibleTarget === Value.false) {
            return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorNonExtensible', P);
        }
        // e. Return undefined.
        return Value.undefined;
    }
    // 12. Let extensibleTarget be ? IsExtensible(target).
    const extensibleTarget = Q(IsExtensible(target));
    // 13. Let resultDesc be ? ToPropertyDescriptor(trapResultObj).
    const resultDesc = Q(ToPropertyDescriptor(trapResultObj));
    // 14. Call CompletePropertyDescriptor(resultDesc).
    CompletePropertyDescriptor(resultDesc);
    // 15. Let valid be IsCompatiblePropertyDescriptor(extensibleTarget, resultDesc, targetDesc).
    const valid = IsCompatiblePropertyDescriptor(extensibleTarget, resultDesc, targetDesc);
    // 16. If valid is false, throw a TypeError exception.
    if (valid === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorIncompatible', P);
    }
    // 17. If resultDesc.[[Configurable]] is false, then
    if (resultDesc.Configurable === Value.false) {
        // a. If targetDesc is undefined or targetDesc.[[Configurable]] is true, then
        if (targetDesc === Value.undefined || targetDesc.Configurable === Value.true) {
            // i. Throw a TypeError exception.
            return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorNonConfigurable', P);
        }
        // b. If resultDesc has a [[Writable]] field and resultDesc.[[Writable]] is false, then
        if ('Writable' in resultDesc && resultDesc.Writable === Value.false) {
            // i. If targetDesc.[[Writable]] is true, throw a TypeError exception.
            if (targetDesc.Writable === Value.true) {
                return surroundingAgent.Throw('TypeError', 'ProxyGetOwnPropertyDescriptorNonConfigurableWritable', P);
            }
        }
    }
    // 18. Return resultDesc.
    return resultDesc;
}

// #sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
function ProxyDefineOwnProperty(P, Desc) {
    const O = this;

    // 1. Assert: IsPropertyKey(P) is true.
    Assert(IsPropertyKey(P));
    // 2. Let handler be O.[[ProxyHandler]].
    const handler = O.ProxyHandler;
    // 3. If handler is null, throw a TypeError exception.
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'defineProperty');
    }
    // 4. Assert: Type(handler) is Object.
    Assert(Type(handler) === 'Object');
    // 5. Let target be O.[[ProxyTarget]].
    const target = O.ProxyTarget;
    // 6. Let trap be ? GetMethod(handler, "defineProperty").
    const trap = Q(GetMethod(handler, new Value('defineProperty')));
    // 7. If trap is undefined, then
    if (trap === Value.undefined) {
        // a. Return ? target.[[DefineOwnProperty]](P, Desc).
        return Q(target.DefineOwnProperty(P, Desc));
    }
    // 8. Let descObj be FromPropertyDescriptor(Desc).
    const descObj = FromPropertyDescriptor(Desc);
    // 9. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P, descObj »)).
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target, P, descObj])));
    // 10. If booleanTrapResult is false, return false.
    if (booleanTrapResult === Value.false) {
        return Value.false;
    }
    // 11. Let targetDesc be ? target.[[GetOwnProperty]](P).
    const targetDesc = Q(target.GetOwnProperty(P));
    // 12. Let extensibleTarget be ? IsExtensible(target).
    const extensibleTarget = Q(IsExtensible(target));
    let settingConfigFalse;
    // 13. If Desc has a [[Configurable]] field and if Desc.[[Configurable]] is false, then
    if (Desc.Configurable !== undefined && Desc.Configurable === Value.false) {
        // a. Let settingConfigFalse be true.
        settingConfigFalse = true;
    } else {
        // Else, let settingConfigFalse be false.
        settingConfigFalse = false;
    }
    // 15. If targetDesc is undefined, then
    if (targetDesc === Value.undefined) {
        // a. If extensibleTarget is false, throw a TypeError exception.
        if (extensibleTarget === Value.false) {
            return surroundingAgent.Throw('TypeError', 'ProxyDefinePropertyNonExtensible', P);
        }
        // b. If settingConfigFalse is true, throw a TypeError exception.
        if (settingConfigFalse === true) {
            return surroundingAgent.Throw('TypeError', 'ProxyDefinePropertyNonConfigurable', P);
        }
    } else {
        // a. If IsCompatiblePropertyDescriptor(extensibleTarget, Desc, targetDesc) is false, throw a TypeError exception.
        if (IsCompatiblePropertyDescriptor(extensibleTarget, Desc, targetDesc) === Value.false) {
            return surroundingAgent.Throw('TypeError', 'ProxyDefinePropertyIncompatible', P);
        }
        // b. If settingConfigFalse is true and targetDesc.[[Configurable]] is true, throw a TypeError exception.
        if (settingConfigFalse === true && targetDesc.Configurable === Value.true) {
            return surroundingAgent.Throw('TypeError', 'ProxyDefinePropertyNonConfigurable', P);
        }
        // c. If IsDataDescriptor(targetDesc) is true, targetDesc.[[Configurable]] is false, and targetDesc.[[Writable]] is true, then
        if (IsDataDescriptor(targetDesc)
            && targetDesc.Configurable === Value.false
            && targetDesc.Writable === Value.true) {
            // i. If Desc has a [[Writable]] field and Desc.[[Writable]] is false, throw a TypeError exception.
            if ('Writable' in Desc && Desc.Writable === Value.false) {
                return surroundingAgent.Throw('TypeError', 'ProxyDefinePropertyNonConfigurableWritable', P);
            }
        }
    }
    return Value.true;
}

// #sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p
function ProxyHasProperty(P) {
    const O = this;

    Assert(IsPropertyKey(P));
    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'has');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('has')));
    if (trap === Value.undefined) {
        return Q(target.HasProperty(P));
    }
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target, P])));
    if (booleanTrapResult === Value.false) {
        const targetDesc = Q(target.GetOwnProperty(P));
        if (targetDesc !== Value.undefined) {
            if (targetDesc.Configurable === Value.false) {
                return surroundingAgent.Throw('TypeError', 'ProxyHasNonConfigurable', P);
            }
            const extensibleTarget = Q(IsExtensible(target));
            if (extensibleTarget === Value.false) {
                return surroundingAgent.Throw('TypeError', 'ProxyHasNonExtensible', P);
            }
        }
    }
    return booleanTrapResult;
}

// #sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
function ProxyGet(P, Receiver) {
    const O = this;

    Assert(IsPropertyKey(P));
    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'get');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('get')));
    if (trap === Value.undefined) {
        return Q(target.Get(P, Receiver));
    }
    const trapResult = Q(Call(trap, handler, [target, P, Receiver]));
    const targetDesc = Q(target.GetOwnProperty(P));
    if (targetDesc !== Value.undefined && targetDesc.Configurable === Value.false) {
        if (IsDataDescriptor(targetDesc) === true && targetDesc.Writable === Value.false) {
            if (SameValue(trapResult, targetDesc.Value) === Value.false) {
                return surroundingAgent.Throw('TypeError', 'ProxyGetNonConfigurableData', P);
            }
        }
        if (IsAccessorDescriptor(targetDesc) === true && targetDesc.Get === Value.undefined) {
            if (trapResult !== Value.undefined) {
                return surroundingAgent.Throw('TypeError', 'ProxyGetNonConfigurableAccessor', P);
            }
        }
    }
    return trapResult;
}

// #sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
function ProxySet(P, V, Receiver) {
    const O = this;

    Assert(IsPropertyKey(P));
    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'set');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('set')));
    if (trap === Value.undefined) {
        return Q(target.Set(P, V, Receiver));
    }
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target, P, V, Receiver])));
    if (booleanTrapResult === Value.false) {
        return Value.false;
    }
    const targetDesc = Q(target.GetOwnProperty(P));
    if (targetDesc !== Value.undefined && targetDesc.Configurable === Value.false) {
        if (IsDataDescriptor(targetDesc) === true && targetDesc.Writable === Value.false) {
            if (SameValue(V, targetDesc.Value) === Value.false) {
                return surroundingAgent.Throw('TypeError', 'ProxySetFrozenData', P);
            }
        }
        if (IsAccessorDescriptor(targetDesc) === true) {
            if (targetDesc.Set === Value.undefined) {
                return surroundingAgent.Throw('TypeError', 'ProxySetFrozenAccessor', P);
            }
        }
    }
    return Value.true;
}

// #sec-proxy-object-internal-methods-and-internal-slots-delete-p
function ProxyDelete(P) {
    const O = this;

    // 1. Assert: IsPropertyKey(P) is true.
    Assert(IsPropertyKey(P));
    // 2. Let handler be O.[[ProxyHandler]].
    const handler = O.ProxyHandler;
    // 3. If handler is null, throw a TypeError exception.
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'deleteProperty');
    }
    // 4. Assert: Type(handler) is Object.
    Assert(Type(handler) === 'Object');
    // 5. Let target be O.[[ProxyTarget]].
    const target = O.ProxyTarget;
    // 6. Let trap be ? GetMethod(handler, "deleteProperty").
    const trap = Q(GetMethod(handler, new Value('deleteProperty')));
    // 7. If trap is undefined, then
    if (trap === Value.undefined) {
        // a. Return ? target.[[Delete]](P).
        return Q(target.Delete(P));
    }
    // 8. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P »)).
    const booleanTrapResult = ToBoolean(Q(Call(trap, handler, [target, P])));
    // 9. If booleanTrapResult is false, return false.
    if (booleanTrapResult === Value.false) {
        return Value.false;
    }
    // 10. Let targetDesc be ? target.[[GetOwnProperty]](P).
    const targetDesc = Q(target.GetOwnProperty(P));
    // 11. If targetDesc is undefined, return true.
    if (targetDesc === Value.undefined) {
        return Value.true;
    }
    // 12. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
    if (targetDesc.Configurable === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxyDeletePropertyNonConfigurable', P);
    }
    // 13. Let extensibleTarget be ? IsExtensible(target).
    const extensibleTarget = Q(IsExtensible(target));
    // 14. If extensibleTarget is false, throw a TypeError exception.
    if (extensibleTarget === Value.false) {
        return surroundingAgent.Throw('TypeError', 'ProxyDeletePropertyNonExtensible', P);
    }
    // 15. Return true.
    return Value.true;
}

// #sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
function ProxyOwnPropertyKeys() {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'ownKeys');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('ownKeys')));
    if (trap === Value.undefined) {
        return Q(target.OwnPropertyKeys());
    }
    const trapResultArray = Q(Call(trap, handler, [target]));
    const trapResult = Q(CreateListFromArrayLike(trapResultArray, ['String', 'Symbol']));
    if (new ValueSet(trapResult).size !== trapResult.length) {
        return surroundingAgent.Throw('TypeError', 'ProxyOwnKeysDuplicateEntries');
    }
    const extensibleTarget = Q(IsExtensible(target));
    const targetKeys = Q(target.OwnPropertyKeys());
    // Assert: targetKeys is a List containing only String and Symbol values.
    // Assert: targetKeys contains no duplicate entries.
    const targetConfigurableKeys = [];
    const targetNonconfigurableKeys = [];
    for (const key of targetKeys) {
        const desc = Q(target.GetOwnProperty(key));
        if (desc !== Value.undefined && desc.Configurable === Value.false) {
            targetNonconfigurableKeys.push(key);
        } else {
            targetConfigurableKeys.push(key);
        }
    }
    if (extensibleTarget === Value.true && targetNonconfigurableKeys.length === 0) {
        return trapResult;
    }
    const uncheckedResultKeys = new ValueSet(trapResult);
    for (const key of targetNonconfigurableKeys) {
        if (!uncheckedResultKeys.has(key)) {
            return surroundingAgent.Throw('TypeError', 'ProxyOwnKeysMissing', 'non-configurable key');
        }
        uncheckedResultKeys.delete(key);
    }
    if (extensibleTarget === Value.true) {
        return trapResult;
    }
    for (const key of targetConfigurableKeys) {
        if (!uncheckedResultKeys.has(key)) {
            return surroundingAgent.Throw('TypeError', 'ProxyOwnKeysMissing', 'configurable key');
        }
        uncheckedResultKeys.delete(key);
    }
    if (uncheckedResultKeys.size > 0) {
        return surroundingAgent.Throw('TypeError', 'ProxyOwnKeysNonExtensible');
    }
    return trapResult;
}

// #sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
function ProxyCall(thisArgument, argumentsList) {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'apply');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    const trap = Q(GetMethod(handler, new Value('apply')));
    if (trap === Value.undefined) {
        return Q(Call(target, thisArgument, argumentsList));
    }
    const argArray = X(CreateArrayFromList(argumentsList));
    return Q(Call(trap, handler, [target, thisArgument, argArray]));
}

// #sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
function ProxyConstruct(argumentsList, newTarget) {
    const O = this;

    const handler = O.ProxyHandler;
    if (handler === Value.null) {
        return surroundingAgent.Throw('TypeError', 'ProxyRevoked', 'construct');
    }
    Assert(Type(handler) === 'Object');
    const target = O.ProxyTarget;
    Assert(IsConstructor(target) === Value.true);
    const trap = Q(GetMethod(handler, new Value('construct')));
    if (trap === Value.undefined) {
        return Q(Construct(target, argumentsList, newTarget));
    }
    const argArray = X(CreateArrayFromList(argumentsList));
    const newObj = Q(Call(trap, handler, [target, argArray, newTarget]));
    if (Type(newObj) !== 'Object') {
        return surroundingAgent.Throw('TypeError', 'NotAnObject', newObj);
    }
    return newObj;
}

export function isProxyExoticObject(O) {
    return 'ProxyHandler' in O;
}

// #sec-proxycreate
export function ProxyCreate(target, handler) {
    // 1. If Type(target) is not Object, throw a TypeError exception.
    if (Type(target) !== 'Object') {
        return surroundingAgent.Throw('TypeError', 'CannotCreateProxyWith', 'non-object', 'target');
    }
    // 2. If Type(handler) is not Object, throw a TypeError exception.
    if (Type(handler) !== 'Object') {
        return surroundingAgent.Throw('TypeError', 'CannotCreateProxyWith', 'non-object', 'handler');
    }
    // 3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
    const P = X(MakeBasicObject(['ProxyHandler', 'ProxyTarget']));
    // 4. Set P's essential internal methods, except for [[Call]] and [[Construct]], to the definitions specified in 9.5.
    P.GetPrototypeOf = ProxyGetPrototypeOf;
    P.SetPrototypeOf = ProxySetPrototypeOf;
    P.IsExtensible = ProxyIsExtensible;
    P.PreventExtensions = ProxyPreventExtensions;
    P.GetOwnProperty = ProxyGetOwnProperty;
    P.DefineOwnProperty = ProxyDefineOwnProperty;
    P.HasProperty = ProxyHasProperty;
    P.Get = ProxyGet;
    P.Set = ProxySet;
    P.Delete = ProxyDelete;
    P.OwnPropertyKeys = ProxyOwnPropertyKeys;
    // 5. If IsCallable(target) is true, then
    if (IsCallable(target) === Value.true) {
        // a. Set P.[[Call]] as specified in #sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist.
        P.Call = ProxyCall;
        // b. If IsConstructor(target) is true, then
        if (IsConstructor(target) === Value.true) {
            // i. Set P.[[Construct]] as specified in #sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget.
            P.Construct = ProxyConstruct;
        }
    }
    // 6. Set P.[[ProxyTarget]] to target.
    P.ProxyTarget = target;
    // 7. Set P.[[ProxyHandler]] to handler.
    P.ProxyHandler = handler;
    // 8. Return P.
    return P;
}
