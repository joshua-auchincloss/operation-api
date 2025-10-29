# array

## Unsized Arrays

Unsized arrays are denoted by pair `[]` tokens after a type.

```pld
type StrArr = str[];
```

## Sized Arrays

Sized arrays are denoted by a pair of `[n]` tokens, where n is the size of the array.

```pld
type Head = u8[128];
```

> [!CAUTION]
> Some generation targets may not support exact sizing - we will pass the metadata wherever possible but due to the constraints of certain languages this is not always type-enforced
